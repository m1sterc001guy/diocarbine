use std::{collections::BTreeMap, str::FromStr, sync::Arc};

use anyhow::bail;
use dioxus::logger::tracing::info;
use fedimint_api_client::api::net::Connector;
use fedimint_bip39::{Bip39RootSecretStrategy, Mnemonic};
use fedimint_client::{
    module_init::ClientModuleInitRegistry, secret::RootSecretStrategy, Client, ClientHandleArc,
};
use fedimint_core::{
    config::FederationId,
    db::{Database, IDatabaseTransactionOpsCoreTyped},
    encoding::Encodable,
    invite_code::InviteCode,
    secp256k1::rand::thread_rng,
    Amount,
};
use fedimint_derive_secret::{ChildId, DerivableSecret};
use fedimint_ln_client::LightningClientInit;
use fedimint_mint_client::MintClientInit;
use fedimint_rocksdb::RocksDb;
use fedimint_wallet_client::WalletClientInit;
use futures_util::StreamExt;

use crate::{
    db::{FederationConfig, FederationConfigKey, FederationConfigKeyPrefix},
    FederationSelector,
};

#[derive(Clone)]
pub(crate) struct Multimint {
    db: Database,
    mnemonic: Mnemonic,
    modules: ClientModuleInitRegistry,
    clients: BTreeMap<FederationId, ClientHandleArc>,
}

impl Multimint {
    pub async fn new() -> anyhow::Result<Self> {
        // TODO: Need android-safe path here
        info!("Opening database...");
        let db: Database = RocksDb::open("client.db").await?.into();

        info!("Generating or reading mnemonic...");
        let mnemonic =
            if let Ok(entropy) = Client::load_decodable_client_secret::<Vec<u8>>(&db).await {
                info!("Loaded mnemonic");
                Mnemonic::from_entropy(&entropy)?
            } else {
                let mnemonic = Bip39RootSecretStrategy::<12>::random(&mut thread_rng());
                info!("Generated mnemonic");

                Client::store_encodable_client_secret(&db, mnemonic.to_entropy()).await?;
                info!("Saved mnemonic");
                mnemonic
            };

        info!("Creating modules...");
        let mut modules = ClientModuleInitRegistry::new();
        modules.attach(LightningClientInit::default());
        modules.attach(MintClientInit);
        modules.attach(WalletClientInit::default());
        modules.attach(fedimint_lnv2_client::LightningClientInit::default());

        info!("Multimint created");
        Ok(Self {
            db,
            mnemonic,
            modules,
            clients: BTreeMap::new(),
        })
    }

    // TODO: Implement recovery
    pub async fn join_federation(&mut self, invite_code: String) -> anyhow::Result<()> {
        let invite_code = InviteCode::from_str(&invite_code)?;
        let federation_id = invite_code.federation_id();
        if self.has_federation(&federation_id).await {
            bail!("Already joined federation")
        }

        let client = self
            .build_client(&federation_id, &invite_code, Connector::Tcp)
            .await?;

        let client_config = Connector::default()
            .download_from_invite_code(&invite_code)
            .await?;
        let federation_config = FederationConfig {
            invite_code,
            connector: Connector::default(),
            federation_name: client_config
                .global
                .federation_name()
                .expect("No federation name")
                .to_owned(),
        };

        self.clients.insert(federation_id, client);
        info!("Added client to internal map");

        let mut dbtx = self.db.begin_transaction().await;
        dbtx.insert_new_entry(
            &FederationConfigKey { id: federation_id },
            &federation_config,
        )
        .await;
        dbtx.commit_tx().await;

        Ok(())
    }

    async fn has_federation(&self, federation_id: &FederationId) -> bool {
        let mut dbtx = self.db.begin_transaction_nc().await;
        dbtx.get_value(&FederationConfigKey { id: *federation_id })
            .await
            .is_some()
    }

    async fn build_client(
        &self,
        federation_id: &FederationId,
        invite_code: &InviteCode,
        connector: Connector,
    ) -> anyhow::Result<ClientHandleArc> {
        info!("Getting client database...");
        let client_db = self.get_client_database(&federation_id);
        info!("Deriving secret...");
        let secret = Self::derive_federation_secret(&self.mnemonic, &federation_id);

        info!("Creating builder...");
        let mut client_builder = Client::builder(client_db).await?;
        client_builder.with_module_inits(self.modules.clone());
        client_builder.with_primary_module_kind(fedimint_mint_client::KIND);
        info!("Created builder");

        if Client::is_initialized(client_builder.db_no_decoders()).await {
            info!("Already initialized, opening...");
            client_builder.open(secret).await
        } else {
            info!("Downloading client config...");
            let client_config = connector.download_from_invite_code(&invite_code).await?;
            info!("Creating client by joining...");
            client_builder
                .join(secret, client_config.clone(), invite_code.api_secret())
                .await
        }
        .map(Arc::new)
    }

    fn get_client_database(&self, federation_id: &FederationId) -> Database {
        let mut prefix = vec![crate::db::DbKeyPrefix::ClientDatabase as u8];
        prefix.append(&mut federation_id.consensus_encode_to_vec());
        self.db.with_prefix(prefix)
    }

    /// Derives a per-federation secret according to Fedimint's multi-federation
    /// secret derivation policy.
    fn derive_federation_secret(
        mnemonic: &Mnemonic,
        federation_id: &FederationId,
    ) -> DerivableSecret {
        let global_root_secret = Bip39RootSecretStrategy::<12>::to_root_secret(mnemonic);
        let multi_federation_root_secret = global_root_secret.child_key(ChildId(0));
        let federation_root_secret = multi_federation_root_secret.federation_key(federation_id);
        let federation_wallet_root_secret = federation_root_secret.child_key(ChildId(0));
        federation_wallet_root_secret.child_key(ChildId(0))
    }

    pub(crate) async fn federations(&self) -> Vec<FederationSelector> {
        let mut dbtx = self.db.begin_transaction_nc().await;
        dbtx.find_by_prefix(&FederationConfigKeyPrefix)
            .await
            .map(|(id, config)| FederationSelector {
                federation_name: config.federation_name,
                federation_id: id.id,
            })
            .collect::<Vec<_>>()
            .await
    }

    pub(crate) async fn balance(&self, federation_id: &FederationId) -> Amount {
        let client = self
            .clients
            .get(federation_id)
            .expect("No federation exists");
        client.get_balance().await
        /*
        let mut dbtx = self.db.begin_transaction_nc().await;
        info!("Getting config...");
        let config = dbtx
            .get_value(&FederationConfigKey { id: *federation_id })
            .await
            .expect("No available config");

        info!("Creating federation client...");
        let client = self
            .build_client(federation_id, &config.invite_code, config.connector)
            .await
            .expect("Could not build client");
        info!("Retrieving balance...");
        client.get_balance().await
        */
    }
}
