# Codecs

Codecs are used as the direct serializable format for ron files. Codecs are then translated into in-memory assets during the loading process for use within the game.

### Defining Codecs

The steps to defining a codec are:
- Create a new struct or enum representing the codec
- Create an associated asset, and implement `From<Codec>` for it
- Create a new `ResourceType` for the asset
- Register your asset with bevy
- Register a `RonAssetLoader<Codec, Asset>` with bevy, which associates the codec with the asset

The asset loader registration can either use discovery, in which case it will automatically load all assets of the specified type in the folder specified by the `ResourceType`, or specific assets to be loaded can be enumerated in the registration.

Codecs should have a `format` field, which allows for updating the format of the codec without breaking compatibility.

### Referencing other assets

Other assets should usually be referenced by `ResourceLocation`. This allows assets to reference one another without introducing any dependency management.

However, if an asset handle is needed for whatever reason, this can be done using the `SystemRegistry` system parameter on a system registered to the `ResolveAssets` startup system set.

If other code needs to hold references to asset handles (for example, it needs access to a collection of dynamic assets and cannot use resource locations), this should be done in the `PopulateAssetRefs` startup system set.

Note that load order outside of these system sets is not guaranteed.

These system sets may also be called at later times if the game needs to reload assets, so ensure that asset references are only stored in resources in these systems to allow for proper updating.