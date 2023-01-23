use super::tx_list::{self, ExpandType};
use crate::log_error;
use async_std::prelude::FutureExt;
use grin_gui_core::{
    config::Config,
    wallet::{Slate, SlateState, Slatepack, TxLogEntry, TxLogEntryType},
};
use grin_gui_widgets::widget::header;
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
    grin_gui_core::wallet::{ContractSetupArgsAPI, StatusMessage, WalletInfo, WalletInterface},
    grin_gui_core::{node::amount_to_hr_string, theme::ColorPalette},
    iced::widget::{button, pick_list, scrollable, text_input, Checkbox, Space},
    iced::{alignment, Alignment, Command, Length},
    serde::{Deserialize, Serialize},
    std::sync::{Arc, RwLock},
};

pub struct StateContainer {
    // pub back_button_state: button::State,
    // pub copy_address_button_state: button::State,
    // pub address_state: text_input::State,
    pub address_value: String,
    // Slatepack read result
    pub slatepack_read_result: String,
    // Actual read slatepack
    pub slatepack_parsed: Option<(Slatepack, Slate)>,
}

impl Default for StateContainer {
    fn default() -> Self {
        Self {
            // back_button_state: Default::default(),
            // copy_address_button_state: Default::default(),
            // address_state: Default::default(),
            address_value: Default::default(),
            slatepack_read_result: localized_string("tx-slatepack-read-result-default"),
            slatepack_parsed: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Action {}

#[derive(Debug, Clone)]
pub enum LocalViewInteraction {
    Back,
    Accept,
    TxAcceptSuccess(Option<String>),
    TxAcceptFailure(Arc<RwLock<Option<anyhow::Error>>>),
}

pub fn handle_message<'a>(
    grin_gui: &mut GrinGui,
    message: LocalViewInteraction,
) -> Result<Command<Message>> {
    let state = &mut grin_gui
        .wallet_state
        .operation_state
        .contract_sign_confirm_state;
    match message {
        LocalViewInteraction::Back => {
            log::debug!("Interaction::WalletOperationContractSignConfirmViewInteraction(Cancel)");
            grin_gui.wallet_state.operation_state.mode =
                crate::gui::element::wallet::operation::Mode::Home;
        }
        LocalViewInteraction::Accept => {
            grin_gui.error.take();

            log::debug!("Interaction::WalletOperationContractSignConfirmViewInteraction(Accept)");
            if state.slatepack_parsed.is_none() {
                log::debug!("you should never see this - dev make sure slatepack is not None");
                return Ok(Command::none());
            }

            let (slatepack, slate) = state.slatepack_parsed.as_ref().unwrap();

            let sp_sending_address = match &slatepack.sender {
                None => "None".to_string(),
                Some(s) => s.to_string(),
            };

            let w = grin_gui.wallet_interface.clone();
            let out_slate = slate.clone();
            let setup_args = ContractSetupArgsAPI {
                // TODO: we should get the confirmation here
                net_change: Some(match slate.state {
                    SlateState::Standard2 => -(slate.amount as i64), // sender net negative
                    // SlateState::Standard3 => slate.amount as i64, // receiver net positive
                    SlateState::Invoice2 => slate.amount as i64, // receiver net positive
                    // SlateState::Invoice3 => slate.amount as i64,  // receiver net positive
                    // TODO: Handle others here. Should probably error if we're at step1
                    _ => slate.amount as i64,
                }),
                // num_participants: num_participants,
                // add_outputs: true,
                ..Default::default()
            };
            println!(
                "will call sign with setup args: slate.state I got: {}, setup_args.net_change: {}",
                slate.state,
                setup_args.net_change.unwrap()
            );
            let fut = move || {
                WalletInterface::sign_contract(w, out_slate, setup_args, sp_sending_address)
            };
            return Ok(Command::perform(fut(), |r| {
                match r.context("Failed to Progress Transaction") {
                    Ok(ret) => Message::Interaction(
                        Interaction::WalletOperationContractSignConfirmViewInteraction(
                            LocalViewInteraction::TxAcceptSuccess(ret),
                        ),
                    ),
                    Err(e) => Message::Interaction(
                        Interaction::WalletOperationContractSignConfirmViewInteraction(
                            LocalViewInteraction::TxAcceptFailure(Arc::new(RwLock::new(Some(e)))),
                        ),
                    ),
                }
            }));
            // match slate.state {
            //     SlateState::Standard1 => {
            //         let fut = move || {
            //             WalletInterface::receive_tx_from_s1(w, out_slate, sp_sending_address)
            //         };

            //         return Ok(Command::perform(fut(), |r| {
            //             match r.context("Failed to Progress Transaction") {
            //                 Ok(ret) => Message::Interaction(
            //                     Interaction::WalletOperationApplyTxConfirmViewInteraction(
            //                         LocalViewInteraction::TxAcceptSuccess(ret),
            //                     ),
            //                 ),
            //                 Err(e) => Message::Interaction(
            //                     Interaction::WalletOperationApplyTxConfirmViewInteraction(
            //                         LocalViewInteraction::TxAcceptFailure(Arc::new(RwLock::new(
            //                             Some(e),
            //                         ))),
            //                     ),
            //                 ),
            //             }
            //         }));
            //     }
            //     SlateState::Standard2 => {
            //         let fut = move || WalletInterface::finalize_from_s2(w, out_slate, true);

            //         return Ok(Command::perform(fut(), |r| {
            //             match r.context("Failed to Progress Transaction") {
            //                 Ok(ret) => Message::Interaction(
            //                     Interaction::WalletOperationApplyTxConfirmViewInteraction(
            //                         LocalViewInteraction::TxAcceptSuccess(ret),
            //                     ),
            //                 ),
            //                 Err(e) => Message::Interaction(
            //                     Interaction::WalletOperationApplyTxConfirmViewInteraction(
            //                         LocalViewInteraction::TxAcceptFailure(Arc::new(RwLock::new(
            //                             Some(e),
            //                         ))),
            //                     ),
            //                 ),
            //             }
            //         }));
            //     }
            //     _ => {
            //         log::error!("Slate state not yet supported");
            //         return Ok(Command::none());
            //     }
            // }
        }
        LocalViewInteraction::TxAcceptSuccess(slate) => {
            log::debug!("{:?}", slate);
            grin_gui
                .wallet_state
                .operation_state
                .contract_sign_success_state
                .encrypted_slate = slate;
            grin_gui.wallet_state.operation_state.mode =
                crate::gui::element::wallet::operation::Mode::ContractSignSuccess;
        }
        LocalViewInteraction::TxAcceptFailure(err) => {
            grin_gui.error = err.write().unwrap().take();
            if let Some(e) = grin_gui.error.as_ref() {
                log_error(e);
            }
        }
    }
    Ok(Command::none())
}

pub fn data_container<'a>(config: &'a Config, state: &'a StateContainer) -> Container<'a, Message> {
    let unit_spacing = 15;

    if state.slatepack_parsed.is_none() {
        return Container::new(Text::new(
            "you should never see this - dev make sure slatepack is not None",
        ));
    }

    // Decode/parse/etc fields for display here
    let (slatepack, slate) = state.slatepack_parsed.as_ref().unwrap();

    let sp_sending_address = match &slatepack.sender {
        None => "None".to_string(),
        Some(s) => s.to_string(),
    };

    let amount = amount_to_hr_string(slate.amount, false);

    let mut state_text = slate.state.to_string();

    // TODO: What's displayed here should change based on the slate state
    let state_text_append = match slate.state {
        SlateState::Standard1 => "You are the recipient - Standard workflow",
        SlateState::Standard2 => {
            "You are the payee, and are finalizing the transaction and sending it to the chain for validation - Standard workflow"
        }
        SlateState::Standard3 => "This transaction is finalised - Standard workflow",
        _ => "Support still in development",
    };

    state_text = format!("{} - {}", state_text, state_text_append);

    let hide_continue =
        slate.state != SlateState::Standard1 && slate.state != SlateState::Standard2;

    // Title row
    let title = Text::new(localized_string("apply-tx-confirm"))
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

    // TX State (i.e. Stage)
    let state_label = Text::new(format!("{}: ", localized_string("tx-state")))
        .size(DEFAULT_FONT_SIZE)
        .horizontal_alignment(alignment::Horizontal::Left);

    let state_label_container =
        Container::new(state_label).style(grin_gui_core::theme::ContainerStyle::NormalBackground);

    let state = Text::new(state_text).size(DEFAULT_FONT_SIZE);
    //.width(Length::Units(400))
    //.style(grin_gui_core::theme::TextInputStyle::AddonsQuery);

    let state_container =
        Container::new(state).style(grin_gui_core::theme::ContainerStyle::NormalBackground);

    let state_row = Row::new().push(state_label_container).push(state_container);

    // Sender address
    let sender_address_label = Text::new(format!("{}: ", localized_string("tx-sender-name")))
        .size(DEFAULT_FONT_SIZE)
        .horizontal_alignment(alignment::Horizontal::Left);

    let sender_address_label_container = Container::new(sender_address_label)
        .style(grin_gui_core::theme::ContainerStyle::NormalBackground);

    let sender_address = Text::new(sp_sending_address).size(DEFAULT_FONT_SIZE);
    //.width(Length::Units(400))
    //.style(grin_gui_core::theme::TextInputStyle::AddonsQuery);

    let sender_address_container = Container::new(sender_address)
        .style(grin_gui_core::theme::ContainerStyle::NormalBackground);

    let sender_address_row = Row::new()
        .push(sender_address_label_container)
        .push(sender_address_container);

    let amount_label = Text::new(format!("{}: ", localized_string("contract-net-change")))
        .size(DEFAULT_FONT_SIZE)
        .horizontal_alignment(alignment::Horizontal::Left);

    let amount_label_container =
        Container::new(amount_label).style(grin_gui_core::theme::ContainerStyle::NormalBackground);

    let amount = Text::new(amount).size(DEFAULT_FONT_SIZE);
    //.width(Length::Units(400))
    //.style(grin_gui_core::theme::TextInputStyle::AddonsQuery);

    let amount_container =
        Container::new(amount).style(grin_gui_core::theme::ContainerStyle::NormalBackground);

    let amount_row = Row::new()
        .push(amount_label_container)
        .push(amount_container);

    let button_height = Length::Units(BUTTON_HEIGHT);
    let button_width = Length::Units(BUTTON_WIDTH);

    let submit_button_label_container =
        Container::new(Text::new(localized_string("tx-continue")).size(DEFAULT_FONT_SIZE))
            .width(button_width)
            .height(button_height)
            .center_x()
            .center_y()
            .align_x(alignment::Horizontal::Center);

    let mut submit_button = Button::new(submit_button_label_container);

    if hide_continue {
        submit_button = submit_button.style(grin_gui_core::theme::ButtonStyle::NormalText);
    } else {
        submit_button = submit_button
            .style(grin_gui_core::theme::ButtonStyle::Primary)
            .on_press(
                Interaction::WalletOperationContractSignConfirmViewInteraction(
                    LocalViewInteraction::Accept,
                ),
            );
    }

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
        .on_press(
            Interaction::WalletOperationContractSignConfirmViewInteraction(
                LocalViewInteraction::Back,
            ),
        )
        .into();

    let submit_container = Container::new(submit_button.map(Message::Interaction)).padding(1);
    let submit_container = Container::new(submit_container)
        .style(grin_gui_core::theme::ContainerStyle::Segmented)
        .padding(1);

    let cancel_container = Container::new(cancel_button.map(Message::Interaction)).padding(1);
    let cancel_container = Container::new(cancel_container)
        .style(grin_gui_core::theme::ContainerStyle::Segmented)
        .padding(1);

    let button_row = Row::new()
        .push(submit_container)
        .push(Space::new(Length::Units(unit_spacing), Length::Units(0)))
        .push(cancel_container);

    let column = Column::new()
        .push(state_row)
        .push(Space::new(Length::Units(0), Length::Units(unit_spacing)))
        .push(sender_address_row)
        .push(Space::new(Length::Units(0), Length::Units(unit_spacing)))
        .push(amount_row)
        .push(Space::new(Length::Units(0), Length::Units(unit_spacing)))
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

    // Returns the final container.
    Container::new(wrapper_column).padding(iced::Padding::from([
        DEFAULT_PADDING, // top
        DEFAULT_PADDING, // right
        DEFAULT_PADDING, // bottom
        DEFAULT_PADDING, // left
    ]))
}
