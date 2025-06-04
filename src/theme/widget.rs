//! Helper functions for creating common widgets.

use std::borrow::Cow;

use bevy::{
    color::palettes::css::*,
    ecs::{spawn::SpawnWith, system::IntoObserverSystem},
    prelude::*,
};

use crate::theme::{interaction::InteractionPalette, palette::*};

/// A root UI node that fills the window and centers its content.
pub fn ui_root(name: impl Into<Cow<'static, str>>) -> impl Bundle {
    (
        Name::new(name),
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(20.0),
            ..default()
        },
        // Don't block picking events for other UI roots.
        Pickable::IGNORE,
    )
}

/// A simple header label. Bigger than [`label`].
pub fn header(text: impl Into<String>) -> impl Bundle {
    (
        Name::new("Header"),
        Text(text.into()),
        TextFont::from_font_size(40.0),
        TextColor(HEADER_TEXT),
    )
}

/// A simple text label.
pub fn label(text: impl Into<String>) -> impl Bundle {
    colored_label(text, LABEL_TEXT)
}

pub fn colored_label(text: impl Into<String>, color: Color) -> impl Bundle {
    let text = text.into();
    (
        Name::new(format!(r#"ColoredLabel("{text}")"#)),
        Text(text),
        TextFont::from_font_size(18.0),
        TextColor(color),
    )
}

/// A large rounded button with text and an action defined as an [`Observer`].
pub fn button<E, B, M, I>(text: impl Into<String>, action: I) -> impl Bundle
where
    E: Event,
    B: Bundle,
    I: IntoObserverSystem<E, B, M>,
{
    button_base(
        text,
        action,
        (
            Node {
                width: Val::Px(300.0),
                height: Val::Px(80.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BorderRadius::MAX,
        ),
        TextFont::from_font_size(40.0),
    )
}

/// A small square button with text and an action defined as an [`Observer`].
pub fn button_small<E, B, M, I>(text: impl Into<String>, action: I) -> impl Bundle
where
    E: Event,
    B: Bundle,
    I: IntoObserverSystem<E, B, M>,
{
    button_base(
        text,
        action,
        Node {
            width: Val::Px(30.0),
            height: Val::Px(30.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        TextFont::from_font_size(40.0),
    )
}

pub fn button_medium<E, B, M, I>(text: impl Into<String>, action: I) -> impl Bundle
where
    E: Event,
    B: Bundle,
    I: IntoObserverSystem<E, B, M>,
{
    button_base(
        text,
        action,
        (
            Node {
                width: Val::Px(90.0),
                height: Val::Px(30.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BorderRadius::all(Val::Px(10.0)),
        ),
        TextFont::from_font_size(16.0),
    )
}

/// A simple button with text and an action defined as an [`Observer`].
/// The button layout is provided by `button_bundle`.
fn button_base<E, B, M, I>(
    text: impl Into<String>,
    action: I,
    button_bundle: impl Bundle,
    font: TextFont,
) -> impl Bundle
where
    E: Event,
    B: Bundle,
    I: IntoObserverSystem<E, B, M>,
{
    let text = text.into();
    let action = IntoObserverSystem::into_system(action);
    (
        Name::new(format!(r#"Button("{text}")"#)),
        Node::default(),
        Children::spawn(SpawnWith(|parent: &mut ChildSpawner| {
            parent
                .spawn((
                    Name::new("Button Inner"),
                    Button,
                    BackgroundColor(BUTTON_BACKGROUND),
                    InteractionPalette {
                        none: BUTTON_BACKGROUND,
                        hovered: BUTTON_HOVERED_BACKGROUND,
                        pressed: BUTTON_PRESSED_BACKGROUND,
                    },
                    children![(
                        Name::new("Button Text"),
                        Text(text),
                        font,
                        TextColor(BUTTON_TEXT),
                        // Don't bubble picking events from the text up to the button.
                        Pickable::IGNORE,
                    )],
                ))
                .insert(button_bundle)
                .observe(action);
        })),
    )
}

pub fn ui_row(children: impl Bundle) -> impl Bundle {
    (
        Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            width: Val::Percent(100.0),
            // height: Val::Percent(100.0),
            padding: UiRect::all(Val::Px(5.0)),
            ..default()
        },
        Pickable::IGNORE,
        children,
    )
}

#[allow(dead_code)]
pub fn ui_column(children: impl Bundle, border: Option<UiRect>) -> impl Bundle {
    (
        Node {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            margin: border.unwrap_or_default(),
            padding: UiRect::all(Val::Px(5.0)),
            ..default()
        },
        Pickable::IGNORE,
        children,
    )
}

#[allow(dead_code)]
pub fn ui_grid(
    grid_columns: Vec<RepeatedGridTrack>,
    grid_rows: Vec<RepeatedGridTrack>,
    children: impl Bundle,
) -> impl Bundle {
    (
        Node {
            display: Display::Grid,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            grid_template_columns: grid_columns,
            grid_template_rows: grid_rows,
            padding: UiRect::all(Val::Px(2.0)),
            ..default()
        },
        Pickable::IGNORE,
        children,
    )
}

pub fn vertical_panel(name: impl Into<String>, children: impl Bundle) -> impl Bundle {
    (
        Name::new(name.into()),
        Node {
            flex_direction: FlexDirection::Column,
            align_content: AlignContent::Start,
            align_items: AlignItems::Start,
            justify_items: JustifyItems::Start,
            justify_content: JustifyContent::Start,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            // row_gap: Val::Px(10.0),
            padding: UiRect::all(Val::Px(5.0)),
            margin: UiRect::all(Val::Px(5.0)),
            ..default()
        },
        Outline {
            width: Val::Px(2.0),
            offset: Val::Px(0.0),
            color: ROSY_BROWN.into(),
        },
        BorderRadius::all(Val::Px(5.0)),
        Pickable::IGNORE,
        children,
    )
}
