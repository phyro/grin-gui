pub mod action_menu;
pub mod chart;
pub mod contract_new;
pub mod contract_new_success;
pub mod contract_sign;
pub mod contract_sign_confirm;
pub mod contract_sign_success;
pub mod home;
pub mod open;
pub mod tx_list;
pub mod tx_list_display;

use {
    crate::gui::{GrinGui, Message},
    crate::Result,
    grin_gui_core::config::Config,
    grin_gui_core::theme::ColorPalette,
    grin_gui_core::theme::{
        Button, Column, Container, Element, PickList, Row, Scrollable, Text, TextInput,
    },
    iced::{Command, Length},
};

pub struct StateContainer {
    pub mode: Mode,
    pub open_state: open::StateContainer,
    pub home_state: home::StateContainer,
    // When changed to true, this should stay false until a wallet is opened with a password
    has_wallet_open_check_failed_one_time: bool,
    // contracts
    pub contract_new_state: contract_new::StateContainer,
    pub contract_new_success_state: contract_new_success::StateContainer,
    pub contract_sign_state: contract_sign::StateContainer,
    pub contract_sign_confirm_state: contract_sign_confirm::StateContainer,
    pub contract_sign_success_state: contract_sign_success::StateContainer,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Mode {
    Open,
    Home,
    // contract
    ContractNew,
    ContractNewSuccess,
    ContractSign,
    ContractSignConfirm,
    ContractSignSuccess,
}

impl Default for StateContainer {
    fn default() -> Self {
        Self {
            mode: Mode::Home,
            open_state: Default::default(),
            home_state: Default::default(),
            has_wallet_open_check_failed_one_time: false,
            // contract
            contract_new_state: Default::default(),
            contract_new_success_state: Default::default(),
            contract_sign_state: Default::default(),
            contract_sign_confirm_state: Default::default(),
            contract_sign_success_state: Default::default(),
        }
    }
}

impl StateContainer {
    pub fn wallet_not_open(&self) -> bool {
        self.has_wallet_open_check_failed_one_time
    }

    pub fn set_wallet_not_open(&mut self) {
        self.has_wallet_open_check_failed_one_time = true;
        self.mode = Mode::Open;
    }

    pub fn clear_wallet_not_open(&mut self) {
        self.has_wallet_open_check_failed_one_time = false;
    }
}

#[derive(Debug, Clone)]
pub enum LocalViewInteraction {}

pub fn handle_message(
    grin_gui: &mut GrinGui,
    message: LocalViewInteraction,
) -> Result<Command<Message>> {
    Ok(Command::none())
}

pub fn data_container<'a>(state: &'a StateContainer, config: &'a Config) -> Container<'a, Message> {
    let content = match state.mode {
        Mode::Open => open::data_container(&state.open_state, config),
        Mode::Home => home::data_container(config, &state.home_state),
        // contracts
        Mode::ContractNew => contract_new::data_container(config, &state.contract_new_state),
        Mode::ContractNewSuccess => {
            contract_new_success::data_container(config, &state.contract_new_success_state)
        }
        Mode::ContractSign => contract_sign::data_container(config, &state.contract_sign_state),
        Mode::ContractSignConfirm => {
            contract_sign_confirm::data_container(config, &state.contract_sign_confirm_state)
        }
        Mode::ContractSignSuccess => {
            contract_sign_success::data_container(config, &state.contract_sign_success_state)
        }
    };

    let column = Column::new().push(content);

    Container::new(column)
        .center_y()
        .center_x()
        .width(Length::Fill)
        .style(grin_gui_core::theme::ContainerStyle::NormalBackground)
}
