use super::tx_list::{self, ExpandType};
use crate::log_error;
use async_std::prelude::FutureExt;
use grin_gui_core::{
    config::Config,
    error::GrinWalletInterfaceError,
    wallet::{TxLogEntry, TxLogEntryType},
};
use grin_gui_widgets::widget::header;

use iced::theme::{self, Theme};
use iced::widget::{checkbox, column, radio, text};
use iced_aw::Card;
use iced_native::Widget;
use std::path::PathBuf;

use super::tx_list::{HeaderState, TxList};

use {
    super::super::super::{
        BUTTON_HEIGHT, BUTTON_WIDTH, DEFAULT_FONT_SIZE, DEFAULT_HEADER_FONT_SIZE, DEFAULT_PADDING,
        SMALLER_FONT_SIZE,
    },
    crate::gui::{GrinGui, Interaction, Message},
    crate::localization::localized_string,
    crate::Result,
    anyhow::Context,
    grin_gui_core::theme::{
        Button, Column, Container, Element, Header, PickList, Row, Scrollable, TableRow, Text,
        TextInput,
    },
    grin_gui_core::wallet::{
        ContractNewArgsAPI, ContractSetupArgsAPI, InitTxArgs, Slate, StatusMessage, WalletInfo,
        WalletInterface,
    },
    grin_gui_core::{
        node::{amount_from_hr_string, amount_to_hr_string},
        theme::{ButtonStyle, ColorPalette, ContainerStyle},
    },
    iced::widget::{button, pick_list, scrollable, text_input, Checkbox, Space},
    iced::{alignment, Alignment, Command, Length},
    serde::{Deserialize, Serialize},
    std::sync::{Arc, RwLock},
};

pub struct StateContainer {
    pub contract_type: ContractType,
    pub counterparty_addr: String,
    // pub sender_addr: String,
    // pub receiver_addr: String,
    pub amount_value: String,
    amount_error: bool,
    slatepack_address_error: bool,
    // pub recipient_address_value: String,
    // // pub amount_input_state: text_input::State,
    // pub amount_value: String,
    // // whether amount has errored
    // amount_error: bool,
    // // slatepack address error
    // slatepack_address_error: bool,
}

impl Default for StateContainer {
    fn default() -> Self {
        Self {
            contract_type: ContractType::Payment,
            counterparty_addr: Default::default(),
            amount_value: Default::default(),
            amount_error: false,
            slatepack_address_error: false,
            //     recipient_address_value: Default::default(),
            //     amount_value: Default::default(),
            //     amount_error: false,
            //     slatepack_address_error: false,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ContractType {
    Payment,
    Invoice,
}

impl ContractType {
    pub const ALL: [ContractType; 2] = [ContractType::Payment, ContractType::Invoice];
}

impl std::fmt::Display for ContractType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                ContractType::Payment => "Payment",
                ContractType::Invoice => "Invoice",
            }
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Action {}

#[derive(Debug, Clone)]
pub enum LocalViewInteraction {
    Back,
    ContractTypeChanged(ContractType),
    CounterpartyAddress(String),
    Amount(String),
    CreateContract(),

    ContractCreatedOk(String),
    ContractCreateError(Arc<RwLock<Option<anyhow::Error>>>),
    SlatepackAddressError,
    // Back,
    // RecipientAddress(String),
    // Amount(String),
    // CreateTransaction(),

    // TxCreatedOk(String),
    // TxCreateError(Arc<RwLock<Option<anyhow::Error>>>),
    // SlatepackAddressError,
}

pub fn handle_message<'a>(
    grin_gui: &mut GrinGui,
    message: LocalViewInteraction,
) -> Result<Command<Message>> {
    let state = &mut grin_gui.wallet_state.operation_state.contract_new_state;

    match message {
        LocalViewInteraction::Back => {
            log::debug!("Interaction::WalletOperationContractNewViewInteraction(Back)");
            grin_gui.wallet_state.operation_state.mode =
                crate::gui::element::wallet::operation::Mode::Home;
        }
        LocalViewInteraction::ContractTypeChanged(ct) => {
            state.contract_type = ct;
        }
        LocalViewInteraction::CounterpartyAddress(s) => {
            state.counterparty_addr = s;
        }
        LocalViewInteraction::Amount(s) => {
            state.amount_value = s;
        }
        LocalViewInteraction::CreateContract() => {
            grin_gui.error.take();
            state.amount_error = false;
            state.slatepack_address_error = false;

            log::debug!("Interaction::WalletOperationContractNewViewInteraction");

            let w = grin_gui.wallet_interface.clone();

            let amount = match amount_from_hr_string(&state.amount_value) {
                // Ok(0) |
                Err(_) => {
                    state.amount_error = true;
                    return Ok(Command::none());
                }
                Ok(a) => a,
            };

            // Todo: Amount parsing + validation, just testing the flow for now
            let mut num_participants = 2;
            // TODO: this is horrible, fix it
            let w2 = w.clone();
            let w2w = w2.write().unwrap();
            // Check if the recipient is us. If it is, the number of participants is 1.
            // TODO: if it's a self-spend, just sign it after and broadcast the tx
            if let Some(o) = &w2w.owner_api {
                let res = o.get_slatepack_address(None, 0)?.to_string();
                if res == state.counterparty_addr.clone() {
                    num_participants = 1;
                }
            }
            let args_ct = ContractNewArgsAPI {
                setup_args: ContractSetupArgsAPI {
                    net_change: Some(match state.contract_type {
                        ContractType::Payment => -(amount as i64),
                        ContractType::Invoice => amount as i64,
                    }),
                    num_participants: num_participants,
                    add_outputs: true,
                    ..Default::default()
                },
                ..Default::default()
            };
            // let args = InitTxArgs {
            //     src_acct_name: None,
            //     amount,
            //     minimum_confirmations: 2,
            //     max_outputs: 500,
            //     num_change_outputs: 1,
            //     selection_strategy_is_use_all: false,
            //     late_lock: Some(false),
            //     ..Default::default()
            // };
            // let fut =
            //     move || WalletInterface::create_tx(w, args, state.recipient_address_value.clone());
            let fut = move || {
                WalletInterface::create_contract(w, args_ct, state.counterparty_addr.clone())
            };

            return Ok(Command::perform(fut(), |r| match r {
                Ok(ret) => {
                    Message::Interaction(Interaction::WalletOperationContractNewViewInteraction(
                        LocalViewInteraction::ContractCreatedOk(ret),
                    ))
                }
                Err(e) => match e {
                    GrinWalletInterfaceError::InvalidSlatepackAddress => Message::Interaction(
                        Interaction::WalletOperationContractNewViewInteraction(
                            LocalViewInteraction::SlatepackAddressError,
                        ),
                    ),
                    _ => Message::Interaction(
                        Interaction::WalletOperationContractNewViewInteraction(
                            LocalViewInteraction::ContractCreateError(Arc::new(RwLock::new(Some(
                                anyhow::Error::from(e),
                            )))),
                        ),
                    ),
                },
            }));
        }
        LocalViewInteraction::ContractCreatedOk(slate) => {
            log::debug!("{:?}", slate);
            grin_gui
                .wallet_state
                .operation_state
                .contract_new_success_state
                .encrypted_slate = slate.to_string();
            grin_gui.wallet_state.operation_state.mode =
                crate::gui::element::wallet::operation::Mode::ContractNewSuccess;
        }
        LocalViewInteraction::ContractCreateError(err) => {
            grin_gui.error = err.write().unwrap().take();
            if let Some(e) = grin_gui.error.as_ref() {
                log_error(e);
            }
        }
        LocalViewInteraction::SlatepackAddressError => state.slatepack_address_error = true,
    }

    Ok(Command::none())
}

pub fn data_container<'a>(config: &'a Config, state: &'a StateContainer) -> Container<'a, Message> {
    // Title row
    let title = Text::new(localized_string("new-contract"))
        .size(DEFAULT_HEADER_FONT_SIZE)
        .horizontal_alignment(alignment::Horizontal::Center);

    let title_container = Container::new(title)
        .style(grin_gui_core::theme::ContainerStyle::BrightBackground)
        .padding(iced::Padding::from([
            2, // top
            0, // right
            2, // bottom
            5, // left
        ]));

    // push more items on to header here: e.g. other buttons, things that belong on the header
    let header_row = Row::new().push(title_container);

    let header_container = Container::new(header_row).padding(iced::Padding::from([
        0,               // top
        0,               // right
        DEFAULT_PADDING, // bottom
        0,               // left
    ]));

    // let checkbox: iced_native::widget::Checkbox<'_, LocalViewInteraction, iced::Renderer> =
    //     checkbox(
    //         "Check me!",
    //         state.contract_type,
    //         LocalViewInteraction::ContractTypeChanged,
    //     );
    let contract_type_list =
        PickList::new(&ContractType::ALL[..], Some(state.contract_type), |l| {
            // Message::Interaction(Interaction::GeneralSettingsViewInteraction(
            //     LocalViewInteraction::ContractTypeChanged(l),
            // ))
            Message::Interaction(Interaction::WalletOperationContractNewViewInteraction(
                LocalViewInteraction::ContractTypeChanged(l),
            ))
            // Message::Interaction(Interaction::WalletOperationContractNewViewInteraction(
            //     LocalViewInteract)ion::ContractTypeChanged(l),
            // )
        })
        .text_size(14)
        .width(Length::Units(120))
        .style(grin_gui_core::theme::PickListStyle::Primary);

    let contract_type_list_container = Container::new(contract_type_list)
        .center_y()
        .width(Length::Units(120))
        .style(grin_gui_core::theme::ContainerStyle::NormalBackground);

    // let choose_theme = [ContractType::Payment, ContractType::Invoice].iter().fold(
    //     column![text("Choose a theme:")].spacing(10),
    //     |column, ct| {
    //         column.push(radio(
    //             format!("{:?}", ct),
    //             *ct,
    //             Some(match state.contract_type {
    //                 ContractType::Payment => ContractType::Payment,
    //                 ContractType::Invoice => ContractType::Invoice,
    //                 // Theme::Custom { .. } => ThemeType::Custom,
    //             }),
    //             LocalViewInteraction::ContractTypeChanged,
    //         ))
    //     },
    // );

    // Sender input
    let counterparty_address = Text::new(localized_string("counterparty-address"))
        .size(DEFAULT_FONT_SIZE)
        .horizontal_alignment(alignment::Horizontal::Left);

    let counterparty_address_container = Container::new(counterparty_address)
        .style(grin_gui_core::theme::ContainerStyle::NormalBackground);

    let counterparty_address_input = TextInput::new("", &state.counterparty_addr, |s| {
        Interaction::WalletOperationContractNewViewInteraction(
            LocalViewInteraction::CounterpartyAddress(s),
        )
    })
    .size(DEFAULT_FONT_SIZE)
    .padding(6)
    .width(Length::Units(400))
    .style(grin_gui_core::theme::TextInputStyle::AddonsQuery);

    let counterparty_address_input: Element<Interaction> = counterparty_address_input.into();

    let payment_info = Text::new(String::from(format!(
        "Position: {}",
        match state.contract_type {
            ContractType::Payment => "Sender",
            ContractType::Invoice => "Receiver",
        }
    )))
    .size(DEFAULT_FONT_SIZE)
    .horizontal_alignment(alignment::Horizontal::Left)
    .style(grin_gui_core::theme::text::TextStyle::Warning);

    let payment_info_container =
        Container::new(payment_info).style(grin_gui_core::theme::ContainerStyle::NormalBackground);

    // // Receiver input
    // let recipient_address = Text::new(localized_string("recipient-address"))
    //     .size(DEFAULT_FONT_SIZE)
    //     .horizontal_alignment(alignment::Horizontal::Left);

    // let recipient_address_container = Container::new(recipient_address)
    //     .style(grin_gui_core::theme::ContainerStyle::NormalBackground);

    // let recipient_address_input = TextInput::new("", &state.receiver_addr, |s| {
    //     Interaction::WalletOperationContractNewViewInteraction(
    //         LocalViewInteraction::ReceiverAddress(s),
    //     )
    // })
    // .size(DEFAULT_FONT_SIZE)
    // .padding(6)
    // .width(Length::Units(400))
    // .style(grin_gui_core::theme::TextInputStyle::AddonsQuery);

    // let recipient_address_input: Element<Interaction> = recipient_address_input.into();

    let address_error = Text::new(localized_string("create-tx-address-error"))
        .size(DEFAULT_FONT_SIZE)
        .horizontal_alignment(alignment::Horizontal::Left)
        .style(grin_gui_core::theme::text::TextStyle::Warning);

    let address_error_container =
        Container::new(address_error).style(grin_gui_core::theme::ContainerStyle::NormalBackground);

    let amount = Text::new(localized_string("create-tx-amount"))
        .size(DEFAULT_FONT_SIZE)
        .horizontal_alignment(alignment::Horizontal::Left);

    let amount_container =
        Container::new(amount).style(grin_gui_core::theme::ContainerStyle::NormalBackground);

    let amount_input = TextInput::new(
        // &mut state.amount_input_state,
        "",
        &state.amount_value,
        |s| Interaction::WalletOperationContractNewViewInteraction(LocalViewInteraction::Amount(s)),
    )
    .size(DEFAULT_FONT_SIZE)
    .padding(6)
    .width(Length::Units(100))
    .style(grin_gui_core::theme::TextInputStyle::AddonsQuery);

    let amount_input: Element<Interaction> = amount_input.into();

    let amount_error = Text::new(localized_string("create-tx-amount-error"))
        .size(DEFAULT_FONT_SIZE)
        .horizontal_alignment(alignment::Horizontal::Left)
        .style(grin_gui_core::theme::text::TextStyle::Warning);

    let amount_error_container =
        Container::new(amount_error).style(grin_gui_core::theme::ContainerStyle::NormalBackground);

    // let address_instruction_container =
    //     Text::new(localized_string("recipient-address-instruction"))
    //         .size(SMALLER_FONT_SIZE)
    //         .horizontal_alignment(alignment::Horizontal::Left);

    // let address_instruction_container2 =
    //     Container::new(address_instruction_container).style(ContainerStyle::NormalBackground);

    let button_height = Length::Units(BUTTON_HEIGHT);
    let button_width = Length::Units(BUTTON_WIDTH);

    let submit_button_label_container =
        Container::new(Text::new(localized_string("tx-create-submit")).size(DEFAULT_FONT_SIZE))
            .width(button_width)
            .height(button_height)
            .center_x()
            .center_y()
            .align_x(alignment::Horizontal::Center);

    let mut submit_button = Button::new(submit_button_label_container)
        .style(grin_gui_core::theme::ButtonStyle::Primary);
    let submit_button =
        submit_button.on_press(Interaction::WalletOperationContractNewViewInteraction(
            LocalViewInteraction::CreateContract(),
        ));
    let submit_button: Element<Interaction> = submit_button.into();

    let cancel_button_label_container =
        Container::new(Text::new(localized_string("cancel")).size(DEFAULT_FONT_SIZE))
            .width(button_width)
            .height(button_height)
            .center_x()
            .center_y()
            .align_x(alignment::Horizontal::Center);

    let cancel_button: Element<Interaction> = Button::new(cancel_button_label_container)
        .style(grin_gui_core::theme::ButtonStyle::Primary)
        .on_press(Interaction::WalletOperationContractNewViewInteraction(
            LocalViewInteraction::Back,
        ))
        .into();

    let submit_container = Container::new(submit_button.map(Message::Interaction)).padding(1);
    let submit_container = Container::new(submit_container)
        .style(grin_gui_core::theme::ContainerStyle::Segmented)
        .padding(1);

    let cancel_container = Container::new(cancel_button.map(Message::Interaction)).padding(1);
    let cancel_container = Container::new(cancel_container)
        .style(grin_gui_core::theme::ContainerStyle::Segmented)
        .padding(1);

    let unit_spacing = 15;
    let button_row = Row::new()
        .push(submit_container)
        .push(Space::new(Length::Units(unit_spacing), Length::Units(0)))
        .push(cancel_container);

    // TODO: Add the sender input here or smth...
    let mut column = Column::new()
        // Choose contract type
        // .push(checkbox)
        // .push(Space::new(Length::Units(0), Length::Units(unit_spacing)))
        .push(contract_type_list_container)
        .push(Space::new(Length::Units(0), Length::Units(5)))
        // sender
        .push(counterparty_address_container)
        .push(Space::new(Length::Units(0), Length::Units(unit_spacing)))
        // .push(address_instruction_container)
        // .push(Space::new(Length::Units(0), Length::Units(unit_spacing)))
        .push(counterparty_address_input.map(Message::Interaction))
        .push(Space::new(Length::Units(0), Length::Units(unit_spacing)))
        .push(payment_info_container)
        .push(Space::new(Length::Units(0), Length::Units(unit_spacing)));
    // receiver
    // .push(recipient_address_container)
    // .push(Space::new(Length::Units(0), Length::Units(unit_spacing)))
    // // .push(address_instruction_container)
    // // .push(Space::new(Length::Units(0), Length::Units(unit_spacing)))
    // .push(recipient_address_input.map(Message::Interaction))
    // .push(Space::new(Length::Units(0), Length::Units(unit_spacing)));

    if state.slatepack_address_error {
        column = column
            .push(address_error_container)
            .push(Space::new(Length::Units(0), Length::Units(unit_spacing)));
    }

    column = column
        .push(amount_container)
        .push(Space::new(Length::Units(0), Length::Units(unit_spacing)))
        .push(amount_input.map(Message::Interaction))
        .push(Space::new(Length::Units(0), Length::Units(unit_spacing)));

    if state.amount_error {
        column = column
            .push(amount_error_container)
            .push(Space::new(Length::Units(0), Length::Units(unit_spacing)));
    }

    column = column
        .push(button_row)
        .push(Space::new(Length::Units(0), Length::Units(unit_spacing)))
        .push(Space::new(
            Length::Units(0),
            Length::Units(unit_spacing + 10),
        ));

    let form_container = Container::new(column)
        .width(Length::Fill)
        .padding(iced::Padding::from([
            0, // top
            0, // right
            0, // bottom
            5, // left
        ]));

    // form container should be scrollable in tiny windows
    let scrollable = Scrollable::new(form_container)
        .height(Length::Fill)
        .style(grin_gui_core::theme::ScrollableStyle::Primary);

    let content = Container::new(scrollable)
        .width(Length::Fill)
        .height(Length::Shrink)
        .style(grin_gui_core::theme::ContainerStyle::NormalBackground);

    let wrapper_column = Column::new()
        .height(Length::Fill)
        .push(header_container)
        .push(content);
    // .push(pick_list);

    // Returns the final container.
    Container::new(wrapper_column).padding(iced::Padding::from([
        DEFAULT_PADDING, // top
        DEFAULT_PADDING, // right
        DEFAULT_PADDING, // bottom
        DEFAULT_PADDING, // left
    ]))
}
