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

# Arrow geometry
ARROW_RADIUS = 330  # centerline radius of the arc body
ARROW_THICKNESS = 68  # stroke width of the arc body
ARROWHEAD_LENGTH = 110  # how far the tip extends past the arc endpoint
ARROWHEAD_HALF_WIDTH = 62  # half-width of the arrowhead base (wider than arc)

# Arc angular spans (Pillow: 0=east, angles increase CW)
# Each arc spans ~150° with ~30° gaps between them.
ARC1_START = 165  # Arc 1: CW from 165° to 315°
ARC1_END = 315
ARC2_START = 345  # Arc 2: CW from 345° to 135°
ARC2_END = 135

POLYGON_SEGMENTS = 200  # smoothness of arc edges


def point_on_circle(cx, cy, r, angle_deg):
    """Point on circle at Pillow angle (0=east, CW)."""
    rad = math.radians(angle_deg)
    return (cx + r * math.cos(rad), cy + r * math.sin(rad))


def draw_curved_arrow(draw, cx, cy, r, thickness, start_deg, end_deg,
                      head_length, head_half_width, color, segments=POLYGON_SEGMENTS):
    """Draw a curved arrow as one filled polygon.

    The arc body runs CW (Pillow convention) from start_deg to end_deg.
    An arrowhead at start_deg points CCW (the visual direction of the arrow).
    """
    half_t = thickness / 2
    inner_r = r - half_t
    outer_r = r + half_t

    # CW angular span of the arc body
    span = end_deg - start_deg
    if span <= 0:
        span += 360

    # Arrowhead tip: on the centerline, past the arc start in the CCW direction
    head_delta_deg = math.degrees(head_length / r)
    tip_angle = start_deg - head_delta_deg
    tip = point_on_circle(cx, cy, r, tip_angle)

    # Wing vertices at the arc start angle
    outer_wing = point_on_circle(cx, cy, r + head_half_width, start_deg)
    inner_wing = point_on_circle(cx, cy, r - head_half_width, start_deg)

    # Build polygon vertices going around the perimeter
    points = []

    # 1. Arrowhead tip
    points.append(tip)

    # 2. Outer wing
    points.append(outer_wing)

    # 3. Outer arc edge (start → end, CW)
    for i in range(segments + 1):
        angle = start_deg + span * i / segments
        points.append(point_on_circle(cx, cy, outer_r, angle))

    # 4. Inner arc edge (end → start, CCW)
    for i in range(segments, -1, -1):
        angle = start_deg + span * i / segments
        points.append(point_on_circle(cx, cy, inner_r, angle))

    # 5. Inner wing
    points.append(inner_wing)

    # Polygon auto-closes back to tip
    draw.polygon(points, fill=color)


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
    inner_margin2 = 20
    draw.ellipse(
        [inner_margin2, inner_margin2, SIZE - 1 - inner_margin2, SIZE - 1 - inner_margin2],
        fill=PURPLE,
    )

    # --- Gold curved arrows (counterclockwise) ---
    draw_curved_arrow(
        draw, CENTER, CENTER, ARROW_RADIUS, ARROW_THICKNESS,
        ARC1_START, ARC1_END, ARROWHEAD_LENGTH, ARROWHEAD_HALF_WIDTH, GOLD,
    )
    draw_curved_arrow(
        draw, CENTER, CENTER, ARROW_RADIUS, ARROW_THICKNESS,
        ARC2_START, ARC2_END, ARROWHEAD_LENGTH, ARROWHEAD_HALF_WIDTH, GOLD,
    )

    # --- Central "R" letter ---
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

    draw.text((CENTER, CENTER), "R", fill=GOLD, font=font, anchor="mm")

    # Save at 1024×1024
    output_path = "icon_1024.png"
    img.save(output_path, "PNG")
    print(f"Saved {output_path}")
    print("Now run: npx tauri icon icon_1024.png -o .")


if __name__ == "__main__":
    main()
