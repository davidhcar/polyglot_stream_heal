use serde::{Deserialize, Serialize};

/// Canonical 19+1 structured visual & prose modalities recognized by the outer
/// deterministic finite-state fence discriminator without regex backtracking.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Modality {
    Markdown,
    Json,
    Mermaid,
    Markmap,
    HtmlSandbox,
    DynamicVisual,
    Voxel3D,
    Canvas2D,
    Canvas3D,
    CanvasHybrid,
    Wave,
    Sim,
    Vector,
    Motion,
    Topology,
    Bi,
    Market,
    Clinical,
    Commerce,
    Actuary,
    MediaCard,
    Whiteboard,
    GitHubBrowser,
    CodeGeneric,
}

impl Modality {
    /// Classifies an opening markdown fence info-string in O(|tag|) time.
    pub fn from_fence_tag(raw_tag: &str) -> Self {
        let primary = raw_tag
            .trim()
            .split(|c: char| c.is_whitespace() || c == ':')
            .next()
            .unwrap_or("")
            .to_ascii_lowercase();

        match primary.as_str() {
            "" => Self::CodeGeneric,
            "json" | "jsonc" | "jsonl" => Self::Json,
            "mermaid" | "flowchart" | "drawio" | "diagram" | "graph" => Self::Mermaid,
            "markmap" | "mindmap" => Self::Markmap,
            "html" | "htm" | "html5" | "webapp" | "pwa" | "app" | "interactive" | "sandbox" => {
                Self::HtmlSandbox
            }
            "dynamic_visual" | "visual_spec" | "kpi_grid" => Self::DynamicVisual,
            "voxel" | "voxels" | "vox" => Self::Voxel3D,
            "canvas2d" | "canvas_2d" => Self::Canvas2D,
            "canvas3d" | "canvas4d" | "canvas_3d" | "canvas_4d" | "3d" | "4d" | "three"
            | "threejs" | "webgl" => Self::Canvas3D,
            "canvashybrid" | "canvas_hybrid" | "hybrid_canvas" => Self::CanvasHybrid,
            "wave" | "harmonic" | "harmonics" | "sine_wave" => Self::Wave,
            "sim" | "simulation" | "physics_sim" | "dynamic" | "dynamics" => Self::Sim,
            "vector" | "biomorphic" | "morphology" => Self::Vector,
            "motion" | "chrono" | "chronophotography" | "stroboscopic" => Self::Motion,
            "topology" | "euler" | "venn" | "chord" => Self::Topology,
            "bi" | "waterfall" | "funnel" | "cohort" | "strategy_quadrant" => Self::Bi,
            "market" | "candlestick" | "orderbook" | "option_payoff" | "yield_curve" => {
                Self::Market
            }
            "clinical" | "ekg" | "kaplan_meier" | "fishbone" | "pedigree" => Self::Clinical,
            "commerce" | "rfm" | "inventory_pareto" | "operations" => Self::Commerce,
            "actuary" | "loss_triangle" | "reinsurance" => Self::Actuary,
            "media-card" | "media_card" | "recipe" | "lab" | "workout" | "guide" => {
                Self::MediaCard
            }
            "whiteboard" | "sketch" => Self::Whiteboard,
            "github-browser" | "github_browser" | "repobrowser" => Self::GitHubBrowser,
            _ => Self::CodeGeneric,
        }
    }

    /// Semantic fallback classifier for unfenced bare payloads.
    pub fn infer_unfenced(raw: &str) -> Self {
        let trimmed = raw.trim_start();
        if trimmed.starts_with('{') || trimmed.starts_with('[') {
            return Self::Json;
        }
        if trimmed.starts_with("<!DOCTYPE")
            || trimmed.starts_with("<!doctype")
            || trimmed.starts_with("<html")
            || trimmed.starts_with("<interactive_app")
            || trimmed.starts_with("<div")
            || trimmed.starts_with("<section")
        {
            return Self::HtmlSandbox;
        }
        let first_word = trimmed
            .split(|c: char| c.is_whitespace() || c == ';' || c == '{' || c == '[')
            .next()
            .unwrap_or("");
        match first_word {
            "flowchart" | "graph" | "sequenceDiagram" | "classDiagram" | "stateDiagram"
            | "stateDiagram-v2" | "erDiagram" | "gantt" | "pie" | "gitGraph" | "C4Context"
            | "timeline" | "quadrantChart" | "architecture" | "architecture-beta" => Self::Mermaid,
            _ => Self::Markdown,
        }
    }

    /// Returns true if the modality carries JSON-structured inner payload grammar.
    #[inline]
    pub fn is_json_backed(self) -> bool {
        matches!(
            self,
            Self::Json
                | Self::DynamicVisual
                | Self::Voxel3D
                | Self::CanvasHybrid
                | Self::Wave
                | Self::Sim
                | Self::Vector
                | Self::Motion
                | Self::Topology
                | Self::Bi
                | Self::Market
                | Self::Clinical
                | Self::Commerce
                | Self::Actuary
                | Self::MediaCard
                | Self::Whiteboard
                | Self::GitHubBrowser
        )
    }
}
