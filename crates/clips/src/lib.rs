//! Editor-core clip primitive: the closed set of placed content types.
//!
//! A clip is a piece of content placed on a track. The *kind* of content is a
//! closed enum — `VideoClip`, `AudioClip`, `TextClip`, `ImageClip`,
//! `VectorClip`, `AdjustmentClip`, `EffectClip`, `CompoundClip` — because these
//! shapes define the project format and every editing consumer depends on them,
//! so they belong to the editor core, not to a plugin. The engine never sees an
//! editor `Clip`; the editor core resolves a clip's properties into compositing
//! primitives (see `architecture.mdc`).
//!
//! Dependency direction: `clips` depends only on `ids` (for `ClipId`, and for
//! `CompoundClip`'s `TimelineId` reference) and `time` (for `RationalTime`). It
//! must NOT depend on `timelines` — `timelines` owns `Track.clips`, so a
//! `clips → timelines` edge would be a cycle. `CompoundClip` reaches "which
//! timeline this black box renders" through the zero-dependency `ids` crate
//! instead of holding a `Timeline` directly.

use ids::{AssetId, ClipId, TimelineId};
use serde::{Deserialize, Serialize};
use time::RationalTime;

/// A clip placed on a track.
///
/// `duration` is always present; it is the clip's length on its track.
/// `start_time` is stored only on *fixed*-layout tracks (overlay/audio), where
/// clips have an explicit position. On the *flow* main track, position is the
/// prefix sum of preceding clip durations, so `start_time` is `None` there —
/// see `architecture.mdc`, "Editor core — Flow/fixed track layouts".
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Clip {
    pub id: ClipId,
    pub kind: ClipKind,
    pub duration: RationalTime,
    pub start_time: Option<RationalTime>,
}

impl Clip {
    /// A clip's end point on its track, in rational time. On a fixed track this
    /// is `start_time + duration`; on a flow track `start_time` is `None` and
    /// the absolute position is only knowable from the track's prefix sum.
    pub fn end_time(&self) -> Option<RationalTime> {
        match self.start_time {
            Some(start) => start.add(self.duration).ok(),
            None => None,
        }
    }
}

/// The closed enum of clip kinds. A `CompoundClip` is a black-box reference to
/// another timeline (see `architecture.mdc`, "Editor core — Compound clips");
/// it is folded into the video track type rather than getting its own because
/// it renders like a video/image clip.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ClipKind {
    Video(VideoClip),
    Audio(AudioClip),
    Image(ImageClip),
    Text(TextClip),
    Vector(VectorClip),
    Adjustment(AdjustmentClip),
    Effect(EffectClip),
    Compound(CompoundClip),
}

/// A clip backed by a media asset (decoded by the engine).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VideoClip {
    pub asset_id: AssetId,
}

/// An audio clip backed by a media asset.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AudioClip {
    pub asset_id: AssetId,
}

/// A still image clip backed by a media asset.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ImageClip {
    pub asset_id: AssetId,
}

/// A text overlay. The editor core resolves `font_size`/`content` into concrete
/// raster primitives at compile time; the engine never sees an editor `Clip`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TextClip {
    pub content: String,
    pub font_size: f32,
}

/// A vector (SVG) overlay — the editor-core name for what the UI calls
/// "Graphics". There is nothing in it that could not be an SVG.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VectorClip {
    pub content: String,
}

/// An adjustment clip: modifies whatever is composited beneath it on the same
/// track (color/levels adjustment). It wraps the accumulator rather than adding
/// a sibling layer — see `architecture.mdc`, "Editor core — Compositor".
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AdjustmentClip {}

/// An effect clip: an open `effect_id` string resolved by the effects registry
/// into a compositing-graph `Effect` node. The resolution gap is not yet closed
/// (see `architecture.mdc`, "Decisions to be made").
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EffectClip {
    pub effect_id: String,
}

/// A compound clip: a black-box reference to another timeline, placed on a
/// track exactly like a video clip. Resolving "render timeline X, feed the
/// composite in as a source" happens entirely within the editor core's render
/// orchestration; the engine never sees an editor `Timeline`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CompoundClip {
    pub timeline_id: TimelineId,
    pub transform: Transform,
    pub opacity: f32,
}

/// A minimal 2D transform carried by `CompoundClip` (and, later, other clips).
/// Self-contained so `clips` needs no `geom` dependency; the editor core maps
/// these into the engine's compositing `Transform` node.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Transform {
    /// Translation in normalized units: (x, y).
    pub translation: (f32, f32),
    /// Scale: (x, y).
    pub scale: (f32, f32),
    /// Rotation in radians.
    pub rotation: f32,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            translation: (0.0, 0.0),
            scale: (1.0, 1.0),
            rotation: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ids::{AssetId, ClipId, TimelineId};

    #[test]
    fn compound_clip_references_timeline_by_id_only() {
        let clip = Clip {
            id: ClipId(1),
            kind: ClipKind::Compound(CompoundClip {
                timeline_id: TimelineId(7),
                transform: Transform::default(),
                opacity: 1.0,
            }),
            duration: RationalTime::new(100, 1).unwrap(),
            start_time: None,
        };
        match clip.kind {
            ClipKind::Compound(c) => assert_eq!(c.timeline_id, TimelineId(7)),
            _ => panic!("expected compound"),
        }
    }

    #[test]
    fn flow_clip_has_no_start_time() {
        let clip = Clip {
            id: ClipId(2),
            kind: ClipKind::Video(VideoClip {
                asset_id: AssetId(3),
            }),
            duration: RationalTime::new(30, 1).unwrap(),
            start_time: None,
        };
        // No stored position on a flow track; end is only derivable from the
        // track's prefix sum.
        assert!(clip.end_time().is_none());
    }
}
