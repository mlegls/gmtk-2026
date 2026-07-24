#[macro_export]
macro_rules! generate_tile_types {
    ($($tile_type:ident),+) => {
        $(
        paste! {
            #[derive(Message, Copy, Clone, Debug)]
            pub struct [<$tile_type EnterMessage>](pub Entity);
            #[derive(Message, Copy, Clone, Debug)]
            pub struct [<$tile_type LeaveMessage>](pub Entity);
        }

        #[derive(Component, Copy, Clone, Default, Debug)]
        pub struct $tile_type;
        )+

        #[derive(Clone, Debug, Hash, PartialEq, Eq)]
        pub enum SpecialTileType {
            $( $tile_type, )+
        }
        pub fn register_messages(app: &mut App) {
            paste! { $(
                app.add_message::<[<$tile_type EnterMessage>]>();
                app.add_message::<[<$tile_type LeaveMessage>]>();
            )+ }
        }
    };
}

#[macro_export]
macro_rules! generate_tile_enter_message_match {
    ($tile_type_var:expr, $entity:expr, $(($tile_type:ident, $tile_type_message:expr)),+) => {
        match $tile_type_var { $(
            crate::ecs::SpecialTileType::$tile_type => {
                paste! {
                    $tile_type_message.write([< $tile_type EnterMessage >]($entity));
                }
            }
        )+ }
    }
}
#[macro_export]
macro_rules! generate_tile_leave_message_match {
    ($tile_type_var:expr, $entity:expr, $(($tile_type:ident, $tile_type_message:expr)),+) => {
        match $tile_type_var { $(
            crate::ecs::SpecialTileType::$tile_type => {
                paste! {
                    $tile_type_message.write([< $tile_type LeaveMessage >]($entity));
                }
            }
        )+ }
    }
}