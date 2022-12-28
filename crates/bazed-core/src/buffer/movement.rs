use xi_rope::Rope;

use super::position::Position;
use crate::{region::Region, user_buffer_op::Motion, view::Viewport, word_boundary};

/// Apply a given motion to a region.
/// if `only_move_head` is false, the tail of the region gets set to the new head,
/// collapsing it into a cursor.
///
/// May result in a region at offset `text.len()`, meaning that it is outside the bounds of the text.
pub(crate) fn apply_motion_to_region(
    text: &Rope,
    vp: &Viewport,
    region: Region,
    only_move_head: bool,
    motion: Motion,
) -> Region {
    // The column the new region wants to be in
    // set when moving vertically, for use when coming out of a shorter line.
    let offset = match motion {
        Motion::Left => text
            .prev_grapheme_offset(region.head)
            .unwrap_or(region.head),
        Motion::Right => text
            .next_grapheme_offset(region.head)
            .unwrap_or(region.head),
        Motion::StartOfLine => text.offset_of_line(text.line_of_offset(region.head)),
        Motion::EndOfLine => {
            let line = text.line_of_offset(region.head);
            let last_line = text.line_of_offset(text.len());
            if line < last_line {
                text.offset_of_line(line + 1)
            } else {
                text.len()
            }
        },
        Motion::NextWordBoundary(boundary_type) => {
            word_boundary::find_word_boundaries(text, region.head)
                .find(|(_, t)| t.matches(&boundary_type))
                .map_or(text.len(), |(offset, _)| offset)
        },
        Motion::PrevWordBoundary(boundary_type) => {
            word_boundary::find_word_boundaries_backwards(text, region.head)
                .find(|(_, t)| t.matches(&boundary_type))
                .map_or(0, |(offset, _)| offset)
        },

        Motion::Up => return move_vertically(text, region, -1, only_move_head),
        Motion::Down => return move_vertically(text, region, 1, only_move_head),
        Motion::TopOfViewport => {
            let current_line = text.line_of_offset(region.head);
            let line_delta = vp.first_line as isize - current_line as isize;
            return move_vertically(text, region, line_delta, only_move_head);
        },
        Motion::BottomOfViewport => {
            let current_line = text.line_of_offset(region.head);
            let line_delta = vp.last_line() as isize - current_line as isize;
            return move_vertically(text, region, line_delta, only_move_head);
        },
    };

    Region {
        head: offset,
        tail: if only_move_head { region.tail } else { offset },
        stickyness: region.stickyness,
        preferred_column: None,
    }
}

/// Move a region vertically by a given number of lines. Preserves all other attributes of the Region.
fn move_vertically(text: &Rope, region: Region, by_lines: isize, only_move_head: bool) -> Region {
    let pos = Position::from_offset(text, region.head).unwrap();
    let pos = match region.preferred_column {
        Some(cur_preferred_column) => pos.with_col(cur_preferred_column),
        None => pos,
    };
    let preferred_column = pos.col;

    let target_line = if by_lines < 0 {
        pos.line.saturating_sub((-by_lines) as usize)
    } else {
        pos.line.saturating_add(by_lines as usize)
    };

    // avoid changing column when the line didn't change for whatever reason
    // (typically due to being at the first or last line)
    let last_line = text.line_of_offset(text.len());
    let offset = if target_line != pos.line {
        pos.with_line(target_line.clamp(0, last_line))
            .to_offset_snapping(text)
    } else {
        region.head
    };

    Region {
        head: offset,
        tail: if only_move_head { region.tail } else { offset },
        preferred_column: Some(preferred_column),
        ..region
    }
}

#[cfg(test)]
mod test {
    use xi_rope::Rope;

    use crate::{
        buffer::movement::apply_motion_to_region, region::Region, test_util,
        user_buffer_op::Motion, view::Viewport, word_boundary::WordBoundaryType,
    };

    #[test]
    fn test_move_next_word_boundary() {
        test_util::setup_test();
        let t = Rope::from("hello hello hello");
        let vp = Viewport::new_ginormeous();
        let motion_start = Motion::NextWordBoundary(WordBoundaryType::Start);
        let motion_end = Motion::NextWordBoundary(WordBoundaryType::End);
        assert_eq!(
            5,
            apply_motion_to_region(&t, &vp, Region::sticky_cursor(1), false, motion_end).head
        );
        assert_eq!(
            6,
            apply_motion_to_region(&t, &vp, Region::sticky_cursor(1), false, motion_start).head
        );
        assert_eq!(
            12,
            apply_motion_to_region(&t, &vp, Region::sticky_cursor(6), false, motion_start).head,
            "Next word boundary should move you, even when starting on a word bounadry",
        );
        assert_eq!(
            17,
            apply_motion_to_region(&t, &vp, Region::sticky_cursor(13), false, motion_end).head,
            "End of the string should be seen as a boundary when moving forwards",
        );
    }

    #[test]
    fn test_move_previous_word_boundary() {
        test_util::setup_test();
        let t = Rope::from("hello hello hello");
        let vp = Viewport::new_ginormeous();
        let motion_start = Motion::PrevWordBoundary(WordBoundaryType::Start);
        let motion_end = Motion::PrevWordBoundary(WordBoundaryType::End);
        assert_eq!(
            0,
            apply_motion_to_region(&t, &vp, Region::sticky_cursor(3), false, motion_start).head,
            "Start of the string should be seen as a boundary when moving backwards",
        );
        assert_eq!(
            0,
            apply_motion_to_region(&t, &vp, Region::sticky_cursor(3), false, motion_start).head,
            "Start of the string should be seen as a boundary when moving backwards",
        );
        assert_eq!(
            5,
            apply_motion_to_region(&t, &vp, Region::sticky_cursor(8), false, motion_end).head
        );
        assert_eq!(
            6,
            apply_motion_to_region(&t, &vp, Region::sticky_cursor(8), false, motion_start).head
        );
        assert_eq!(
            0,
            apply_motion_to_region(&t, &vp, Region::sticky_cursor(6), false, motion_start).head
        );
    }
}
