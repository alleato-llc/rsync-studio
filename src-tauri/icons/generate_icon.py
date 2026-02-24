"""Generate the Rsync Studio app icon.

Purple circle background with gold counterclockwise arrows and a gold "R" in the center.

Usage:
    python3 generate_icon.py

Then run `npx tauri icon icon_1024.png -o .` from this directory to generate
all platform-specific icon variants (icns, ico, sized PNGs).
"""

import math

from PIL import Image, ImageDraw, ImageFont

SIZE = 1024
CENTER = SIZE // 2

# Colors
PURPLE = (124, 58, 237, 255)  # #7C3AED
PURPLE_DARK = (109, 40, 217, 255)  # #6D28D9
GOLD = (245, 158, 11, 255)  # #F59E0B
TRANSPARENT = (0, 0, 0, 0)

# Arrow arc parameters
ARROW_RADIUS = 355  # radius of arrow centerline
ARROW_THICKNESS = 68  # stroke width of arcs
ARROWHEAD_LENGTH = 90  # length of arrowhead triangle
ARROWHEAD_HALF_WIDTH = 52  # half-width of arrowhead base

# Arc angular parameters (Pillow angles: 0=east, CW, degrees)
# Gap centers at ~330° (1 o'clock) and ~150° (7 o'clock)
ARC1_START_ANGLE = 165  # Arc 1 drawn CW from 165° to 315°
ARC1_END_ANGLE = 315
ARC2_START_ANGLE = 345  # Arc 2 drawn CW from 345° to 135°
ARC2_END_ANGLE = 135

# The arrowhead is at the START of the Pillow CW arc (= the CCW leading edge)
ARROW1_TIP_ANGLE = 165  # degrees (Pillow convention)
ARROW2_TIP_ANGLE = 345


def point_on_circle(cx, cy, r, angle_deg):
    """Get point on circle at given Pillow angle (CW from east)."""
    rad = math.radians(angle_deg)
    return (cx + r * math.cos(rad), cy + r * math.sin(rad))


def ccw_tangent(angle_deg):
    """Unit tangent vector in the CCW direction at given Pillow angle."""
    rad = math.radians(angle_deg)
    # CCW = decreasing Pillow angle direction
    # d/d(-θ) of (cos θ, sin θ) = (sin θ, -cos θ)
    tx = math.sin(rad)
    ty = -math.cos(rad)
    return (tx, ty)


def draw_arrowhead(draw, cx, cy, r, angle_deg, length, half_width, color):
    """Draw a triangular arrowhead at the given position pointing CCW."""
    # Point on circle
    px, py = point_on_circle(cx, cy, r, angle_deg)

    # CCW tangent direction
    tx, ty = ccw_tangent(angle_deg)

    # Normal (perpendicular to tangent, pointing outward/inward)
    nx, ny = -ty, tx  # rotated 90° CW from tangent

    # Arrowhead tip: offset forward along tangent
    tip_x = px + length * tx
    tip_y = py + length * ty

    # Arrowhead base points: offset back and to each side
    base_cx = px - (length * 0.2) * tx  # slightly behind the arc endpoint
    base_cy = py - (length * 0.2) * ty

    base1_x = base_cx + half_width * nx
    base1_y = base_cy + half_width * ny
    base2_x = base_cx - half_width * nx
    base2_y = base_cy - half_width * ny

    draw.polygon([(tip_x, tip_y), (base1_x, base1_y), (base2_x, base2_y)], fill=color)


def main():
    img = Image.new("RGBA", (SIZE, SIZE), TRANSPARENT)
    draw = ImageDraw.Draw(img)

    # --- Purple circular background ---
    draw.ellipse([0, 0, SIZE - 1, SIZE - 1], fill=PURPLE)

    # Subtle inner ring for depth
    inner_margin = 12
    draw.ellipse(
        [inner_margin, inner_margin, SIZE - 1 - inner_margin, SIZE - 1 - inner_margin],
        fill=PURPLE_DARK,
    )
    # Slightly lighter inner fill
    inner_margin2 = 20
    draw.ellipse(
        [
            inner_margin2,
            inner_margin2,
            SIZE - 1 - inner_margin2,
            SIZE - 1 - inner_margin2,
        ],
        fill=PURPLE,
    )

    # --- Counterclockwise arrow arcs (gold) ---
    arc_bbox = [
        CENTER - ARROW_RADIUS,
        CENTER - ARROW_RADIUS,
        CENTER + ARROW_RADIUS,
        CENTER + ARROW_RADIUS,
    ]

    # Arc 1: CW from 165° to 315° in Pillow (visually covers top-left quadrants)
    draw.arc(
        arc_bbox, ARC1_START_ANGLE, ARC1_END_ANGLE, fill=GOLD, width=ARROW_THICKNESS
    )

    # Arc 2: CW from 345° to 135° in Pillow (visually covers bottom-right quadrants)
    draw.arc(
        arc_bbox, ARC2_START_ANGLE, ARC2_END_ANGLE, fill=GOLD, width=ARROW_THICKNESS
    )

    # --- Arrowheads ---
    # Arrow 1 tip at 165° (pointing CCW toward ~150° direction)
    draw_arrowhead(
        draw,
        CENTER,
        CENTER,
        ARROW_RADIUS,
        ARROW1_TIP_ANGLE,
        ARROWHEAD_LENGTH,
        ARROWHEAD_HALF_WIDTH,
        GOLD,
    )

    # Arrow 2 tip at 345° (pointing CCW toward ~330° direction)
    draw_arrowhead(
        draw,
        CENTER,
        CENTER,
        ARROW_RADIUS,
        ARROW2_TIP_ANGLE,
        ARROWHEAD_LENGTH,
        ARROWHEAD_HALF_WIDTH,
        GOLD,
    )

    # --- Central "R" letter ---
    # Verdana Bold, fall back to Helvetica/Arial
    font_size = 380
    try:
        font = ImageFont.truetype(
            "/System/Library/Fonts/Supplemental/Verdana Bold.ttf", font_size
        )
    except Exception:
        try:
            font = ImageFont.truetype(
                "/System/Library/Fonts/HelveticaNeue.ttc", font_size, index=8
            )
        except Exception:
            font = ImageFont.truetype("/Library/Fonts/Arial.ttf", font_size)

    # Draw text centered
    draw.text((CENTER, CENTER), "R", fill=GOLD, font=font, anchor="mm")

    # Save at 1024x1024
    output_path = "icon_1024.png"
    img.save(output_path, "PNG")
    print(f"Saved {output_path}")
    print("Now run: npx tauri icon icon_1024.png -o .")


if __name__ == "__main__":
    main()
