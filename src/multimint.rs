use std::{collections::BTreeMap, str::FromStr, sync::Arc};

use anyhow::bail;
use fedimint_api_client::api::net::Connector;
use fedimint_bip39::{Bip39RootSecretStrategy, Mnemonic};
use fedimint_client::{module_init::ClientModuleInitRegistry, secret::RootSecretStrategy, Client, ClientBuilder, ClientHandleArc};
use fedimint_core::{config::FederationId, db::{Database, IDatabaseTransactionOpsCoreTyped}, encoding::Encodable, invite_code::InviteCode, secp256k1::rand::thread_rng};
use fedimint_derive_secret::{ChildId, DerivableSecret};
use fedimint_ln_client::LightningClientInit;
use fedimint_mint_client::MintClientInit;
use fedimint_wallet_client::WalletClientInit;
use futures_util::StreamExt;

use crate::db::{FederationConfig, FederationConfigKey, FederationConfigKeyPrefix, Redb};

#[derive(Clone)]
pub(crate) struct Multimint {
    db: Database,
    pub(crate) clients: BTreeMap<FederationId, ClientHandleArc>,
    mnemonic: Mnemonic,
    modules: ClientModuleInitRegistry,
}

impl Multimint {
    pub async fn new() -> anyhow::Result<Self> {
        // TODO: Need android-safe path here
        let db: Database = Redb::open("fedimint.redb")?.into();

        let mnemonic = if let Ok(entropy) = Client::load_decodable_client_secret::<Vec<u8>>(&db).await {
                Mnemonic::from_entropy(&entropy)?
            } else {
                let mnemonic = Bip39RootSecretStrategy::<12>::random(&mut thread_rng());

                Client::store_encodable_client_secret(&db, mnemonic.to_entropy())
                    .await?;
                mnemonic
            };

        let mut modules = ClientModuleInitRegistry::new();
        modules.attach(LightningClientInit::default());
        modules.attach(MintClientInit);
        modules.attach(WalletClientInit::default());
        modules.attach(fedimint_lnv2_client::LightningClientInit::default()); 

        let mut multimint = Self {
            db,
            clients: BTreeMap::new(),
            mnemonic,
            modules,
        };
        multimint.load_clients().await?;

        Ok(multimint)
    }

    // TODO: Implement recovery
    pub async fn join_federation(&mut self, invite_code: String) -> anyhow::Result<()> {
        let invite_code = InviteCode::from_str(&invite_code)?;
        let federation_id = invite_code.federation_id();
        if self.has_federation(&federation_id) {
            bail!("Already joined federation")
        }

        let federation_config = FederationConfig {
            invite_code,
            connector: Connector::default(),
        };

        let client = self.build_client(&federation_id, &federation_config).await?;
        self.clients.insert(federation_id, client);

        let mut dbtx = self.db.begin_transaction().await;
        dbtx.insert_new_entry(&FederationConfigKey { id: federation_id }, &federation_config).await;
        dbtx.commit_tx().await;
        
        Ok(())
    }

    fn has_federation(&self, federation_id: &FederationId) -> bool {
        self.clients.contains_key(federation_id)
    }

    async fn build_client(&self, federation_id: &FederationId, config: &FederationConfig) -> anyhow::Result<ClientHandleArc> {
        let client_db = self.get_client_database(&federation_id);
        let secret = Self::derive_federation_secret(&self.mnemonic, &federation_id);

        let mut client_builder = Client::builder(client_db).await?;
        client_builder.with_module_inits(self.modules.clone());
        client_builder.with_primary_module_kind(fedimint_mint_client::KIND);

        if Client::is_initialized(client_builder.db_no_decoders()).await {
            client_builder.open(secret).await
        } else {
            let client_config = config
                .connector
                .download_from_invite_code(&config.invite_code)
                .await?;
            client_builder
                .join(secret, client_config.clone(), config.invite_code.api_secret())
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

    async fn load_clients(&mut self) -> anyhow::Result<()> {

        let mut dbtx = self.db.begin_transaction_nc().await;
        let configs = dbtx.find_by_prefix(&FederationConfigKeyPrefix).await.collect::<BTreeMap<_, _>>().await;

        for (federation_id, federation_config) in configs {
            let client = self.build_client(&federation_id.id, &federation_config).await?;
            self.clients.insert(federation_id.id, client);
        }

        Ok(())
    }
}