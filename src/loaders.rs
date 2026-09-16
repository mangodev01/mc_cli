use crate::version::FabricLike;

pub const FABRIC: FabricLike = FabricLike {
	game_versions: "https://meta.fabricmc.net/v2/versions/game",
	loader_versions: "https://meta.fabricmc.net/v2/versions/loader",
	intermediary_versions: "https://meta.fabricmc.net/v2/versions/intermediary",
	maven: "https://maven.fabricmc.net/",
	id: "fabric"
};

pub const QUILT: FabricLike = FabricLike {
	game_versions: "https://meta.quiltmc.org/v3/versions/game",
	loader_versions: "https://meta.quiltmc.org/v3/versions/loader",
	intermediary_versions: "https://meta.quiltmc.org/v3/versions/intermediary",
	maven: "https://maven.quiltmc.org/",
	id: "quilt"
};

pub const LABRIC: FabricLike = FabricLike {
	game_versions: "https://meta.legacyfabric.net/v2/versions/game",
	loader_versions: "https://meta.legacyfabric.net/v2/versions/loader",
	intermediary_versions: "https://meta.legacyfabric.net/v2/versions/intermediary",
	maven: "https://repo.legacyfabric.net/legacyfabric/",
	id: "labric"
};

pub const BABRIC: FabricLike = FabricLike {
	game_versions: "https://meta.babric.glass-launcher.net/v2/versions/game",
	loader_versions: "https://meta.babric.glass-launcher.net/v2/versions/loader",
	intermediary_versions: "https://meta.babric.glass-launcher.net/v2/versions/intermediary",
	maven: "https://maven.glass-launcher.net/babric/",
	id: "babric"
};

pub const ORNITHE: FabricLike = FabricLike {
	game_versions: "https://meta.ornithemc.net/v3/versions/gen2/game",
	loader_versions: "https://meta.ornithemc.net/v3/versions/gen2/fabric-loader",
	intermediary_versions: "https://meta.ornithemc.net/v3/versions/gen2/intermediary",
	maven: "https://maven.ornithemc.net/",
	id: "ornithe"
};
