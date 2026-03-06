use crate::{error::SvgError, vector::asset::SvgVectorAsset};
use bevy::{
    asset::{AssetLoader, LoadContext, io::Reader},
    prelude::*,
    tasks::ConditionalSendFuture,
};

/// The [`AssetLoader`] for [`SvgVectorAsset`]s.
///
/// Loads an [`SVG`](https://en.wikipedia.org/wiki/SVG) file into an
/// [`SvgVectorAsset`] containing both the original data and parsed [`Tree`](resvg::usvg::Tree).
#[derive(Default, TypePath)]
pub struct SvgVectorAssetLoader;

impl AssetLoader for SvgVectorAssetLoader {
    type Asset = SvgVectorAsset;
    type Settings = ();
    type Error = SvgError;

    fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext,
    ) -> impl ConditionalSendFuture<Output = Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            let mut buf = Vec::new();
            reader.read_to_end(&mut buf).await?;
            SvgVectorAsset::new(buf)
        })
    }
}
