/// Placeholder for all wallet calls
/// Should eventually feature async calls that work via local wallet or remote owner API
use grin_wallet::cmd::wallet_args::inst_wallet;
use grin_wallet_api::{Foreign, Owner};
use grin_wallet_config::{self, GlobalWalletConfig};
use grin_wallet_controller::command::InitArgs;
use grin_wallet_impls::DefaultLCProvider;
use grin_wallet_libwallet::{NodeClient, WalletInst, WalletLCProvider};

pub use grin_core::global;
use grin_core::{self};
use grin_keychain as keychain;
use grin_util::{file, Mutex, ZeroingString};

use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use dirs;

// Re-exports
pub use global::ChainTypes;
pub use grin_wallet_impls::HTTPNodeClient;
pub use grin_wallet_libwallet::contract::can_finalize;
pub use grin_wallet_libwallet::contract::types::{ContractNewArgsAPI, ContractSetupArgsAPI};
pub use grin_wallet_libwallet::{
    InitTxArgs, RetrieveTxQueryArgs, RetrieveTxQuerySortOrder, Slate, SlateState, Slatepack,
    SlatepackAddress, StatusMessage, TxLogEntry, TxLogEntryType, WalletInfo,
};

use crate::error::GrinWalletInterfaceError;
use crate::logger;

use std::convert::TryFrom;

/// Wallet configuration file name
pub const WALLET_CONFIG_FILE_NAME: &str = "grin-wallet.toml";

const WALLET_LOG_FILE_NAME: &str = "grin-wallet.log";

const GRIN_HOME: &str = ".grin";
/// Wallet data directory
pub const GRIN_WALLET_DIR: &str = "wallet_data";
/// Wallet top level directory
pub const GRIN_WALLET_TOP_LEVEL_DIR: &str = "grin_wallet";
/// Wallet top level directory
pub const GRIN_WALLET_DEFAULT_DIR: &str = "default";
/// Node API secret
pub const API_SECRET_FILE_NAME: &str = ".foreign_api_secret";
/// Owner API secret
pub const OWNER_API_SECRET_FILE_NAME: &str = ".owner_api_secret";

/// TODO - this differs from the default directory in 5.x,
/// need to reconcile this with existing installs somehow

pub fn get_grin_wallet_default_path(chain_type: &global::ChainTypes) -> PathBuf {
    // Check if grin dir exists
    let mut grin_path = match dirs::home_dir() {
        Some(p) => p,
        None => PathBuf::new(),
    };
    grin_path.push(GRIN_HOME);
    grin_path.push(chain_type.shortname());
    grin_path.push(GRIN_WALLET_TOP_LEVEL_DIR);
    grin_path.push(GRIN_WALLET_DEFAULT_DIR);

    grin_path
}

pub fn create_grin_wallet_path(chain_type: &global::ChainTypes, sub_dir: &str) -> PathBuf {
    // Check if grin dir exists
    let mut grin_path = match dirs::home_dir() {
        Some(p) => p,
        None => PathBuf::new(),
    };
    grin_path.push(GRIN_HOME);
    grin_path.push(chain_type.shortname());
    grin_path.push(GRIN_WALLET_TOP_LEVEL_DIR);
    grin_path.push(sub_dir);

    grin_path
}

pub type WalletInterfaceHttpNodeClient = WalletInterface<
    DefaultLCProvider<'static, HTTPNodeClient, keychain::ExtKeychain>,
    HTTPNodeClient,
>;

pub struct WalletInterface<L, C>
where
    L: WalletLCProvider<'static, C, keychain::ExtKeychain> + 'static,
    C: NodeClient + 'static + Clone,
{
    pub chain_type: Option<grin_core::global::ChainTypes>,
    pub config: Option<GlobalWalletConfig>,
    // owner api will hold instantiated/opened wallets
    pub owner_api: Option<Owner<L, C, keychain::ExtKeychain>>,
    // Also need reference to foreign API
    pub foreign_api: Option<Foreign<'static, L, C, keychain::ExtKeychain>>,
    // Simple flag to check whether wallet has been opened
    wallet_is_open: bool,
    // Hold on to check node foreign API secret for now
    pub check_node_foreign_api_secret_path: Option<String>,
    // Whether to use embedded node for check node
    use_embedded_node: bool,

    node_client: C,
}

impl<L, C> WalletInterface<L, C>
where
    L: WalletLCProvider<'static, C, keychain::ExtKeychain>,
    C: NodeClient + 'static + Clone,
{
    pub fn new(node_client: C) -> Self {
        WalletInterface {
            chain_type: None,
            config: None,
            owner_api: None,
            foreign_api: None,
            wallet_is_open: false,
            check_node_foreign_api_secret_path: None,
            node_client,
            use_embedded_node: true,
        }
    }

    fn set_chain_type(&mut self, chain_type: global::ChainTypes) {
        self.chain_type = Some(chain_type);
    }

    pub fn set_check_node_foreign_api_secret_path(&mut self, secret: &str) {
        self.check_node_foreign_api_secret_path = Some(secret.to_owned())
    }

    pub fn config_exists(&self, path: &str) -> bool {
        grin_wallet_config::config_file_exists(&path)
    }

    pub fn default_config_exists(&self) -> bool {
        match self.chain_type {
            Some(chain_type) => {
                self.config_exists(get_grin_wallet_default_path(&chain_type).to_str().unwrap())
            }
            _ => false,
        }
    }

    pub fn wallet_is_open(&self) -> bool {
        self.wallet_is_open
    }

    pub fn set_use_embedded_node(
        wallet_interface: Arc<RwLock<WalletInterface<L, C>>>,
        value: bool,
    ) {
        let mut w = wallet_interface.write().unwrap();
        if w.use_embedded_node != value {
            w.owner_api = None;
        }
        w.use_embedded_node = value;
    }

    /// Sets the top level directory of the wallet and creates default config if config
    /// doesn't already exist. The initial config is created based off of the chain type.
    fn inst_wallet(
        wallet_interface: Arc<RwLock<WalletInterface<L, C>>>,
        chain_type: global::ChainTypes,
        top_level_directory: PathBuf,
    ) -> Result<
        Arc<Mutex<Box<dyn WalletInst<'static, L, C, keychain::ExtKeychain>>>>,
        GrinWalletInterfaceError,
    > {
        let mut w = wallet_interface.write().unwrap();
        // path for config file
        let data_path = Some(top_level_directory.clone());

        // creates default config file for chain type at data path
        let config =
            grin_wallet_config::initial_setup_wallet(&chain_type, data_path, true).unwrap();

        // Update logging config
        let mut logging_config = config.members.as_ref().unwrap().logging.clone().unwrap();
        logging_config.tui_running = Some(false);
        logger::update_logging_config(logger::LogArea::Wallet, logging_config);

        let wallet_config = config.clone().members.unwrap().wallet;

        // Set node client address and Foreign API Secret if needed
        if w.use_embedded_node {
            w.node_client
                .set_node_url(&wallet_config.check_node_api_http_addr);

            let check_node_secret =
                file::get_first_line(w.check_node_foreign_api_secret_path.clone());
            w.node_client.set_node_api_secret(check_node_secret);
        }

        let wallet_inst =
            inst_wallet(wallet_config.clone(), w.node_client.clone()).unwrap_or_else(|e| {
                println!("{}", e);
                std::process::exit(1);
            });

        {
            let mut wallet_lock = wallet_inst.lock();
            let lc = wallet_lock.lc_provider().unwrap();
            // set top level directory
            lc.set_top_level_directory(top_level_directory.to_str().unwrap());
        }

        w.config = Some(config);

        Ok(wallet_inst)
    }

    fn inst_apis(
        wallet_interface: Arc<RwLock<WalletInterface<L, C>>>,
        chain_type: global::ChainTypes,
        top_level_directory: PathBuf,
    ) -> Result<(), GrinWalletInterfaceError> {
        let wallet_inst = WalletInterface::inst_wallet(
            wallet_interface.clone(),
            chain_type,
            top_level_directory,
        )?;
        let mut w = wallet_interface.write().unwrap();
        w.owner_api = Some(Owner::new(wallet_inst.clone(), None));
        w.foreign_api = Some(Foreign::new(wallet_inst.clone(), None, None, false));
        global::set_local_chain_type(chain_type);

        Ok(())
    }

    pub async fn init(
        wallet_interface: Arc<RwLock<WalletInterface<L, C>>>,
        password: String,
        top_level_directory: PathBuf,
        display_name: String,
        chain_type: global::ChainTypes,
        recovery_phrase: Option<String>,
    ) -> Result<(String, String, String, global::ChainTypes), GrinWalletInterfaceError> {
        WalletInterface::inst_apis(
            wallet_interface.clone(),
            chain_type,
            top_level_directory.clone(),
        )?;

        let w = wallet_interface.read().unwrap();

        let recover_length = recovery_phrase.clone().map(|f| f.len()).unwrap_or(32);
        let recover_phrase = recovery_phrase.map(|f| ZeroingString::from(f));

        let args = InitArgs {
            list_length: recover_length,
            password: password.clone().into(),
            config: w.config.clone().unwrap().clone().members.unwrap().wallet,
            recovery_phrase: recover_phrase.clone(),
            restore: recover_phrase.is_some(),
        };

        let (tld, ret_phrase) = match w.owner_api.as_ref() {
            Some(o) => {
                let tld = {
                    let mut w_lock = o.wallet_inst.lock();
                    let p = w_lock.lc_provider()?;
                    let logging_config = w
                        .config
                        .clone()
                        .unwrap()
                        .clone()
                        .members
                        .unwrap()
                        .logging
                        .clone();

                    log::debug!(
                        "core::wallet::InitWallet Top Level Directory: {:?}",
                        p.get_top_level_directory(),
                    );

                    p.create_config(
                        &chain_type,
                        WALLET_CONFIG_FILE_NAME,
                        Some(args.config),
                        logging_config,
                        None,
                    )?;

                    p.create_wallet(
                        None,
                        args.recovery_phrase,
                        args.list_length,
                        args.password.clone(),
                        chain_type == global::ChainTypes::Testnet,
                    )?;

                    p.get_top_level_directory()?
                };

                (tld, o.get_mnemonic(None, args.password)?.to_string())
            }
            None => ("".to_string(), "".to_string()),
        };

        Ok((tld, ret_phrase, display_name, chain_type))
    }

    pub async fn open_wallet(
        wallet_interface: Arc<RwLock<WalletInterface<L, C>>>,
        password: String,
        top_level_directory: PathBuf,
        chain_type: global::ChainTypes,
    ) -> Result<(), GrinWalletInterfaceError> {
        WalletInterface::inst_apis(
            wallet_interface.clone(),
            chain_type,
            top_level_directory.clone(),
        )?;

        let mut w = wallet_interface.write().unwrap();

        if let Some(o) = &w.owner_api {
            // ignoring secret key
            let _ = o.open_wallet(None, password.into(), false)?;
            // Start the updater
            o.start_updater(None, std::time::Duration::from_secs(60))?;
            w.wallet_is_open = true;
            // set wallet interface chain type
            w.set_chain_type(chain_type);
            return Ok(());
        } else {
            return Err(GrinWalletInterfaceError::OwnerAPINotInstantiated);
        }
    }

    pub async fn close_wallet(
        wallet_interface: Arc<RwLock<WalletInterface<L, C>>>,
    ) -> Result<(), GrinWalletInterfaceError> {
        let mut w = wallet_interface.write().unwrap();
        if let Some(o) = &w.owner_api {
            o.close_wallet(None);
            w.wallet_is_open = false;
            return Ok(());
        } else {
            return Err(GrinWalletInterfaceError::OwnerAPINotInstantiated);
        }
    }

    pub fn get_wallet_updater_status(
        wallet_interface: Arc<RwLock<WalletInterface<L, C>>>,
    ) -> Result<Vec<StatusMessage>, GrinWalletInterfaceError> {
        let w = wallet_interface.read().unwrap();
        if let Some(o) = &w.owner_api {
            let res = o.get_updater_messages(1)?;
            return Ok(res);
        } else {
            return Err(GrinWalletInterfaceError::OwnerAPINotInstantiated);
        }
    }

    pub fn encrypt_slatepack(
        api: &Owner<L, C, keychain::ExtKeychain>,
        dest: &str,
        unenc_slate: &Slate,
    ) -> Result<String, GrinWalletInterfaceError> {
        let address = match SlatepackAddress::try_from(dest) {
            Ok(a) => Some(a),
            Err(_) => return Err(GrinWalletInterfaceError::InvalidSlatepackAddress),
        };
        // encrypt for recipient by default
        let recipients = match address.clone() {
            Some(a) => vec![a],
            None => vec![],
        };
        Ok(api.create_slatepack_message(None, &unenc_slate, Some(0), recipients)?)
    }

    /// Attempt to decode and decrypt a given slatepack
    pub fn decrypt_slatepack(
        wallet_interface: Arc<RwLock<WalletInterface<L, C>>>,
        slatepack: String,
    ) -> Result<(Slatepack, Slate), GrinWalletInterfaceError> {
        let w = wallet_interface.read().unwrap();
        if let Some(o) = &w.owner_api {
            let sp = o.decode_slatepack_message(None, slatepack.clone(), vec![0])?;
            let slate = o.slate_from_slatepack_message(None, slatepack, vec![0])?;
            return Ok((sp, slate));
        } else {
            return Err(GrinWalletInterfaceError::OwnerAPINotInstantiated);
        }
    }

    pub async fn get_wallet_info(
        wallet_interface: Arc<RwLock<WalletInterface<L, C>>>,
    ) -> Result<(bool, WalletInfo), GrinWalletInterfaceError> {
        let w = wallet_interface.read().unwrap();
        if let Some(o) = &w.owner_api {
            let res = o.retrieve_summary_info(None, false, 2)?;
            return Ok(res);
        } else {
            return Err(GrinWalletInterfaceError::OwnerAPINotInstantiated);
        }
    }

    pub async fn get_txs(
        wallet_interface: Arc<RwLock<WalletInterface<L, C>>>,
        query_args: Option<RetrieveTxQueryArgs>,
    ) -> Result<(bool, Vec<TxLogEntry>), GrinWalletInterfaceError> {
        let w = wallet_interface.read().unwrap();
        if let Some(o) = &w.owner_api {
            let res = o.retrieve_txs(None, true, None, None, query_args)?;
            /*for tx in &mut res.1 {
                if tx.amount_credited == 0 && tx.amount_debited == 0 {
                    let saved_tx = o.get_stored_tx(None, Some(tx.id), None);
                    if let Ok(st) = saved_tx {
                        // Todo: have to check more things here, this is just for tx display
                        if let Some(s) = st {
                            println!("TWO: {}", s.amount);
                            tx.amount_debited = s.amount;
                        }
                    }
                }
            };*/
            return Ok(res);
        } else {
            return Err(GrinWalletInterfaceError::OwnerAPINotInstantiated);
        }
    }

    pub async fn get_slatepack_address(
        wallet_interface: Arc<RwLock<WalletInterface<L, C>>>,
    ) -> Result<String, GrinWalletInterfaceError> {
        let w = wallet_interface.read().unwrap();
        if let Some(o) = &w.owner_api {
            let res = o.get_slatepack_address(None, 0)?;
            return Ok(res.to_string());
        } else {
            return Err(GrinWalletInterfaceError::OwnerAPINotInstantiated);
        }
    }

    // contracts
    pub async fn create_contract(
        wallet_interface: Arc<RwLock<WalletInterface<L, C>>>,
        init_args: ContractNewArgsAPI,
        dest_slatepack_address: String,
    ) -> Result<String, GrinWalletInterfaceError> {
        let w = wallet_interface.write().unwrap();
        let _address = match SlatepackAddress::try_from(dest_slatepack_address.as_str()) {
            Ok(a) => Some(a),
            Err(_) => return Err(GrinWalletInterfaceError::InvalidSlatepackAddress),
        };
        if let Some(o) = &w.owner_api {
            // let slate = { o.init_send_tx(None, init_args)? };
            let slate = { o.contract_new(None, &init_args)? };
            // No need to lock outputs as they're already locked atomically
            // o.tx_lock_outputs(None, &slate)?;
            return WalletInterface::encrypt_slatepack(o, &dest_slatepack_address, &slate);
        } else {
            return Err(GrinWalletInterfaceError::OwnerAPINotInstantiated);
        }
    }
    pub async fn sign_contract(
        wallet_interface: Arc<RwLock<WalletInterface<L, C>>>,
        slate: Slate,
        setup_args: ContractSetupArgsAPI,
        dest_slatepack_address: String,
        // dest_slatepack_address: String,
    ) -> Result<Option<String>, GrinWalletInterfaceError> {
        let w = wallet_interface.write().unwrap();
        if let Some(o) = &w.owner_api {
            // let slate = { o.init_send_tx(None, init_args)? };
            let slate = { o.contract_sign(None, &slate, &setup_args)? };
            // No need to lock outputs as they're already locked atomically
            // o.tx_lock_outputs(None, &slate)?;
            let is_finalized = can_finalize(&slate);
            if is_finalized {
                o.post_tx(None, &slate, true)?;
                return Ok(None);
            } else {
                let encrypted =
                    WalletInterface::encrypt_slatepack(o, &dest_slatepack_address, &slate)?;
                return Ok(Some(encrypted));
            }
            // return WalletInterface::encrypt_slatepack(o, &dest_slatepack_address, &slate);
        } else {
            return Err(GrinWalletInterfaceError::OwnerAPINotInstantiated);
        }
    }

    pub async fn create_tx(
        wallet_interface: Arc<RwLock<WalletInterface<L, C>>>,
        init_args: InitTxArgs,
        dest_slatepack_address: String,
    ) -> Result<String, GrinWalletInterfaceError> {
        let w = wallet_interface.write().unwrap();
        let _address = match SlatepackAddress::try_from(dest_slatepack_address.as_str()) {
            Ok(a) => Some(a),
            Err(_) => return Err(GrinWalletInterfaceError::InvalidSlatepackAddress),
        };
        if let Some(o) = &w.owner_api {
            let slate = { o.init_send_tx(None, init_args)? };
            o.tx_lock_outputs(None, &slate)?;
            return WalletInterface::encrypt_slatepack(o, &dest_slatepack_address, &slate);
        } else {
            return Err(GrinWalletInterfaceError::OwnerAPINotInstantiated);
        }
    }

    pub async fn receive_tx_from_s1(
        wallet_interface: Arc<RwLock<WalletInterface<L, C>>>,
        slate: Slate,
        dest_slatepack_address: String,
    ) -> Result<Option<String>, GrinWalletInterfaceError> {
        let w = wallet_interface.write().unwrap();
        let ret_slate;
        if let Some(f) = &w.foreign_api {
            ret_slate = f.receive_tx(&slate, None, None)?;
        } else {
            return Err(GrinWalletInterfaceError::ForeignAPINotInstantiated);
        }
        if let Some(o) = &w.owner_api {
            let encrypted =
                WalletInterface::encrypt_slatepack(o, &dest_slatepack_address, &ret_slate)?;
            return Ok(Some(encrypted));
        } else {
            return Err(GrinWalletInterfaceError::OwnerAPINotInstantiated);
        }
    }

    pub async fn finalize_from_s2(
        wallet_interface: Arc<RwLock<WalletInterface<L, C>>>,
        slate: Slate,
        send_to_chain: bool,
    ) -> Result<Option<String>, GrinWalletInterfaceError> {
        let w = wallet_interface.write().unwrap();
        if let Some(o) = &w.owner_api {
            let ret_slate = o.finalize_tx(None, &slate)?;
            o.post_tx(None, &ret_slate, true)?;
            return Ok(None);
        } else {
            return Err(GrinWalletInterfaceError::ForeignAPINotInstantiated);
        }
    }

    pub async fn cancel_tx(
        wallet_interface: Arc<RwLock<WalletInterface<L, C>>>,
        id: u32,
    ) -> Result<u32, GrinWalletInterfaceError> {
        let w = wallet_interface.write().unwrap();
        if let Some(o) = &w.owner_api {
            o.cancel_tx(None, Some(id), None)?;
            return Ok(id);
        } else {
            return Err(GrinWalletInterfaceError::OwnerAPINotInstantiated);
        }
    }

    /*pub async fn tx_lock_outputs(
        wallet_interface: Arc<RwLock<WalletInterface<L, C>>>,
        init_args: InitTxArgs,
    ) -> Result<Slate, GrinWalletInterfaceError> {
        let w = wallet_interface.write().unwrap();
        if let Some(o) = &w.owner_api {
            let slate = {
                o.init_send_tx(None, init_args)?
            };
            return Ok(slate);
        } else {
            return Err(GrinWalletInterfaceError::OwnerAPINotInstantiated);
        }
    }*/

    /*pub async fn get_recovery_phrase(wallet_interface: Arc<RwLock<WalletInterface<L, C>>>, password: String) -> String {
        let mut w = wallet_interface.read().unwrap();
        w.owner_api.get_mnemonic(name, password.into())
    }*/
}
