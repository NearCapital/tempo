pub mod ceremony;
pub(crate) mod manager;

#[derive(Debug, Clone, Copy)]
pub enum HardforkRegime {
    PreAllegretto,
    PostAllegretto,
}
