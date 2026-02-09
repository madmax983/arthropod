# Figma Visual/Style Properties Reference

Comprehensive reference of all visual and rendering properties from the Figma REST API and Plugin API.
Covers both the REST API wire format (JSON responses) and the Plugin API TypeScript types.

Sources:
- [Figma REST API Spec (api_types.ts)](https://github.com/figma/rest-api-spec/blob/main/dist/api_types.ts)
- [Figma Plugin API Data Types](https://developers.figma.com/docs/plugins/api/data-types/)
- [Figma REST API File Property Types](https://developers.figma.com/docs/rest-api/file-property-types/)
- [Figma Plugin API Paint](https://developers.figma.com/docs/plugins/api/Paint/)
- [Figma Plugin API TextNode](https://developers.figma.com/docs/plugins/api/TextNode/)
- [Figma Plugin API VectorPath](https://developers.figma.com/docs/plugins/api/VectorPath/)
- [Figma Plugin API VectorNetwork](https://developers.figma.com/docs/plugins/api/VectorNetwork/)

---

## Table of Contents

1. [Color Types](#1-color-types)
2. [Paint Types](#2-paint-types)
3. [Stroke Properties](#3-stroke-properties)
4. [Effect Types](#4-effect-types)
5. [Corner Radius](#5-corner-radius)
6. [Blend Modes](#6-blend-modes)
7. [Opacity](#7-opacity)
8. [Clipping and Masks](#8-clipping-and-masks)
9. [Constraints and Layout](#9-constraints-and-layout)
10. [Text Properties](#10-text-properties)
11. [Vector / Path Properties](#11-vector--path-properties)
12. [Image Fill Properties](#12-image-fill-properties)
13. [Transform](#13-transform)

---

## 1. Color Types

### RGBA (REST API)

```
RGBA {
  r: number   // 0..1
  g: number   // 0..1
  b: number   // 0..1
  a: number   // 0..1
}
```

### RGB (Plugin API - used in SolidPaint)

```
RGB {
  r: number   // 0..1
  g: number   // 0..1
  b: number   // 0..1
}
```

Note: In the Plugin API, SolidPaint uses `RGB` (no alpha) and controls transparency
via the paint-level `opacity` field. In the REST API, `RGBA` is used throughout.

---

## 2. Paint Types

Paint is a union type representing how fills and strokes are rendered.

```
Paint = SolidPaint | GradientPaint | ImagePaint | VideoPaint | PatternPaint
```

### Common Paint Properties (all paint types inherit these)

| Field           | Type        | Default   | Description                            |
|-----------------|-------------|-----------|----------------------------------------|
| `visible`       | `boolean`   | `true`    | Whether this paint is visible          |
| `opacity`       | `number`    | `1.0`     | Overall paint opacity (0..1)           |
| `blendMode`     | `BlendMode` | `NORMAL`  | How this paint blends with layers below|

### 2.1 SolidPaint

| Field           | Type       | Description                              |
|-----------------|------------|------------------------------------------|
| `type`          | `"SOLID"`  | Paint type discriminant                  |
| `color`         | `RGBA`     | REST API: RGBA with alpha                |
|                 | `RGB`      | Plugin API: RGB without alpha (use opacity)|
| `boundVariables`| `object`   | Variable bindings (e.g., `{ color: VariableAlias }`) |

### 2.2 GradientPaint

| Field                     | Type           | Description                                       |
|---------------------------|----------------|---------------------------------------------------|
| `type`                    | `enum`         | One of the gradient types (see below)              |
| `gradientHandlePositions` | `Vector[]`     | REST API: 3 handle positions defining the gradient |
| `gradientTransform`       | `Transform`    | Plugin API: 2x3 affine transform matrix            |
| `gradientStops`           | `ColorStop[]`  | Array of color stops along the gradient            |

**Gradient type values:**

| Value                | Description                          |
|----------------------|--------------------------------------|
| `GRADIENT_LINEAR`    | Linear gradient between two points   |
| `GRADIENT_RADIAL`    | Radial gradient from center outward  |
| `GRADIENT_ANGULAR`   | Angular/conic gradient around a point|
| `GRADIENT_DIAMOND`   | Diamond-shaped gradient              |

**ColorStop:**

| Field           | Type       | Description                           |
|-----------------|------------|---------------------------------------|
| `position`      | `number`   | Position along gradient (0..1)        |
| `color`         | `RGBA`     | Color at this stop (includes alpha)   |
| `boundVariables`| `object`   | Variable bindings for the color       |

**Gradient Handle Positions (REST API):**
- 3 `Vector` points: `[start, end, widthControl]`
- `start` and `end` define the gradient line
- `widthControl` defines perpendicular width (for radial/diamond)

### 2.3 ImagePaint

| Field            | Type           | Default | Description                                      |
|------------------|----------------|---------|--------------------------------------------------|
| `type`           | `"IMAGE"`      |         | Paint type discriminant                          |
| `scaleMode`      | `enum`         |         | How the image is scaled (see below)              |
| `imageRef`       | `string`       |         | REST API: image reference hash                   |
| `imageHash`      | `string\|null` |         | Plugin API: image hash (null if not uploaded)     |
| `imageTransform` | `Transform`    |         | 2x3 affine matrix (used with CROP mode)          |
| `scalingFactor`  | `number`       |         | Scale multiplier (used with TILE mode)           |
| `rotation`       | `number`       | `0`     | Rotation in degrees (90-degree increments)       |
| `filters`        | `ImageFilters` |         | Image adjustment filters                         |
| `gifRef`         | `string`       |         | REST API: GIF reference (if animated)            |

**ScaleMode values:**

| Value    | Description                                           |
|----------|-------------------------------------------------------|
| `FILL`   | Image scales to fill the entire layer (may crop)      |
| `FIT`    | Image scales to fit within the layer (may letterbox)  |
| `CROP`   | Image is cropped, positioned via `imageTransform`     |
| `TILE`   | Image tiles/repeats, scaled by `scalingFactor`        |
| `STRETCH`| REST API only: image stretches to fill bounds         |

**ImageFilters:**

| Field         | Type     | Range        | Default | Description                |
|---------------|----------|--------------|---------|----------------------------|
| `exposure`    | `number` | `-1.0..1.0`  | `0.0`   | Exposure adjustment        |
| `contrast`    | `number` | `-1.0..1.0`  | `0.0`   | Contrast adjustment        |
| `saturation`  | `number` | `-1.0..1.0`  | `0.0`   | Saturation adjustment      |
| `temperature` | `number` | `-1.0..1.0`  | `0.0`   | Color temperature shift    |
| `tint`        | `number` | `-1.0..1.0`  | `0.0`   | Tint shift                 |
| `highlights`  | `number` | `-1.0..1.0`  | `0.0`   | Highlights adjustment      |
| `shadows`     | `number` | `-1.0..1.0`  | `0.0`   | Shadows adjustment         |

### 2.4 VideoPaint

Same structure as ImagePaint but with:

| Field            | Type     | Description                    |
|------------------|----------|--------------------------------|
| `type`           | `"VIDEO"`| Paint type discriminant        |
| `videoHash`      | `string` | Video content hash             |

### 2.5 PatternPaint (Beta)

| Field                 | Type       | Description                                |
|-----------------------|------------|--------------------------------------------|
| `type`                | `"PATTERN"`| Paint type discriminant                    |
| `sourceNodeId`        | `string`   | Node used as the pattern tile source       |
| `tileType`            | `enum`     | `RECTANGULAR`, `HORIZONTAL_HEXAGONAL`, `VERTICAL_HEXAGONAL` |
| `scalingFactor`       | `number`   | Scale of the pattern                       |
| `spacing`             | `Vector`   | Spacing between tiles `{ x, y }`          |
| `horizontalAlignment` | `enum`     | `START`, `CENTER`, `END`                   |
| `verticalAlignment`   | `enum`     | `START`, `CENTER`, `END`                   |

---

## 3. Stroke Properties

### Stroke Paint Array

Strokes are defined as an array of `Paint` objects (same types as fills).

```
strokes: Paint[]
```

### Stroke Geometry Properties

| Field              | Type                  | Default    | Description                                      |
|--------------------|-----------------------|------------|--------------------------------------------------|
| `strokeWeight`     | `number`              |            | Uniform stroke weight in pixels                  |
| `individualStrokeWeights` | `StrokeWeights` |            | Per-side stroke weights (frames only)            |
| `strokeAlign`      | `enum`                | `CENTER`   | Where stroke is drawn relative to path           |
| `strokeCap`        | `enum`                | `NONE`     | Shape at open ends of paths                      |
| `strokeJoin`       | `enum`                | `MITER`    | Shape at path segment joints                     |
| `strokeDashes`     | `number[]`            | `[]`       | Dash pattern (alternating dash/gap lengths)      |
| `strokeMiterAngle` | `number`              |            | Angle threshold for miter-to-bevel fallback      |

**StrokeWeights (per-side, frames only):**

| Field    | Type     | Description       |
|----------|----------|-------------------|
| `top`    | `number` | Top stroke weight |
| `right`  | `number` | Right stroke weight|
| `bottom` | `number` | Bottom stroke weight|
| `left`   | `number` | Left stroke weight|

**StrokeAlign values:**

| Value     | Description                                         |
|-----------|-----------------------------------------------------|
| `INSIDE`  | Stroke drawn inside the path boundary               |
| `OUTSIDE` | Stroke drawn outside the path boundary              |
| `CENTER`  | Stroke centered on the path boundary                |

**StrokeCap values:**

| Value              | Description                              |
|--------------------|------------------------------------------|
| `NONE`             | Flat/butt cap (no extension)             |
| `ROUND`            | Semicircular cap                         |
| `SQUARE`           | Square cap extending beyond endpoint     |
| `LINE_ARROW`       | Line arrow cap                           |
| `TRIANGLE_ARROW`   | Triangle arrow cap                       |
| `DIAMOND_FILLED`   | Filled diamond cap                       |
| `CIRCLE_FILLED`    | Filled circle cap                        |
| `TRIANGLE_FILLED`  | Filled triangle cap                      |
| `WASHI_TAPE_1..6`  | Decorative washi tape caps (6 variants)  |

**StrokeJoin values:**

| Value   | Description                                      |
|---------|--------------------------------------------------|
| `MITER` | Sharp corner (extends to a point)                |
| `BEVEL` | Flat corner (cuts the point off)                 |
| `ROUND` | Rounded corner                                   |

---

## 4. Effect Types

Effects are stored as an ordered array. Multiple effects stack.

```
effects: Effect[]
```

```
Effect = DropShadowEffect | InnerShadowEffect | BlurEffect
```

### 4.1 Drop Shadow

| Field                   | Type        | Default | Description                                    |
|-------------------------|-------------|---------|------------------------------------------------|
| `type`                  | `"DROP_SHADOW"` |     | Effect type discriminant                       |
| `visible`               | `boolean`   | `true`  | Whether the effect is visible                  |
| `color`                 | `RGBA`      |         | Shadow color (with alpha for transparency)     |
| `blendMode`             | `BlendMode` |         | How the shadow blends with content below       |
| `offset`                | `Vector`    |         | Shadow offset `{ x, y }` in pixels            |
| `radius`                | `number`    |         | Blur radius in pixels (Gaussian sigma)         |
| `spread`                | `number`    | `0`     | Expansion of the shadow shape in pixels        |
| `showShadowBehindNode`  | `boolean`   |         | Whether shadow renders behind the node itself  |
| `boundVariables`        | `object`    |         | Variable bindings for `radius`, `spread`, `color`, `offsetX`, `offsetY` |

### 4.2 Inner Shadow

| Field           | Type               | Default | Description                              |
|-----------------|--------------------|---------|------------------------------------------|
| `type`          | `"INNER_SHADOW"`   |         | Effect type discriminant                 |
| `visible`       | `boolean`          | `true`  | Whether the effect is visible            |
| `color`         | `RGBA`             |         | Shadow color (with alpha)                |
| `blendMode`     | `BlendMode`        |         | How the shadow blends                    |
| `offset`        | `Vector`           |         | Shadow offset `{ x, y }` in pixels      |
| `radius`        | `number`           |         | Blur radius in pixels                    |
| `spread`        | `number`           | `0`     | Expansion of the shadow shape            |
| `boundVariables`| `object`           |         | Variable bindings                        |

### 4.3 Layer Blur

| Field     | Type              | Default    | Description                              |
|-----------|-------------------|------------|------------------------------------------|
| `type`    | `"LAYER_BLUR"`    |            | Effect type discriminant                 |
| `visible` | `boolean`         | `true`     | Whether the effect is visible            |
| `radius`  | `number`          |            | Blur radius in pixels                    |
| `blurType`| `enum`            | `"NORMAL"` | `NORMAL` or `PROGRESSIVE`               |

**Progressive blur additional fields** (when `blurType` is `PROGRESSIVE`):

| Field         | Type     | Description                              |
|---------------|----------|------------------------------------------|
| `startRadius` | `number` | Starting blur radius                     |
| `startOffset` | `Vector` | Start position of the progressive blur   |
| `endOffset`   | `Vector` | End position of the progressive blur     |

### 4.4 Background Blur

| Field     | Type                  | Default    | Description                              |
|-----------|-----------------------|------------|------------------------------------------|
| `type`    | `"BACKGROUND_BLUR"`   |            | Effect type discriminant                 |
| `visible` | `boolean`             | `true`     | Whether the effect is visible            |
| `radius`  | `number`              |            | Blur radius in pixels                    |
| `blurType`| `enum`                | `"NORMAL"` | `NORMAL` or `PROGRESSIVE`               |

Same progressive blur sub-fields apply as with LAYER_BLUR.

---

## 5. Corner Radius

### HasCornerTrait

| Field                  | Type        | Default | Description                                      |
|------------------------|-------------|---------|--------------------------------------------------|
| `cornerRadius`         | `number`    | `0`     | Uniform corner radius for all corners            |
| `cornerSmoothing`      | `number`    | `0`     | iOS-style smooth corners (0..1, 0.6 = iOS default)|
| `rectangleCornerRadii` | `number[4]` |         | Per-corner radii: `[topLeft, topRight, bottomRight, bottomLeft]` |

Notes:
- When `rectangleCornerRadii` is set, it overrides `cornerRadius`.
- In the Plugin API, `cornerRadius` returns `figma.mixed` if corners differ.
- `cornerSmoothing` creates "squircle" curves (like iOS app icons) instead of circular arcs.

---

## 6. Blend Modes

All blend modes available for nodes, paints, and effects:

### Standard Blend Modes

| Value           | Category    | Description                                        |
|-----------------|-------------|----------------------------------------------------|
| `PASS_THROUGH`  | Special     | Node-only (not for paints). Children blend directly with content below the group. |
| `NORMAL`        | Normal      | Default. Source over destination.                  |

### Darken Group

| Value           | Description                                            |
|-----------------|--------------------------------------------------------|
| `DARKEN`        | Keeps the darker of source and destination             |
| `MULTIPLY`      | Multiplies source and destination colors               |
| `LINEAR_BURN`   | Adds source and destination, then subtracts white      |
| `COLOR_BURN`    | Darkens destination to reflect source                  |

### Lighten Group

| Value           | Description                                            |
|-----------------|--------------------------------------------------------|
| `LIGHTEN`       | Keeps the lighter of source and destination            |
| `SCREEN`        | Inverse multiply (lightens)                            |
| `LINEAR_DODGE`  | Adds source and destination colors                     |
| `COLOR_DODGE`   | Brightens destination to reflect source                |

### Contrast Group

| Value           | Description                                            |
|-----------------|--------------------------------------------------------|
| `OVERLAY`       | Multiply or Screen depending on destination brightness |
| `SOFT_LIGHT`    | Subtle version of Overlay                              |
| `HARD_LIGHT`    | Multiply or Screen depending on source brightness      |

### Inversion Group

| Value           | Description                                            |
|-----------------|--------------------------------------------------------|
| `DIFFERENCE`    | Absolute difference between source and destination     |
| `EXCLUSION`     | Similar to Difference but lower contrast               |

### Component Group

| Value           | Description                                            |
|-----------------|--------------------------------------------------------|
| `HUE`           | Source hue, destination saturation and luminosity       |
| `SATURATION`    | Source saturation, destination hue and luminosity       |
| `COLOR`         | Source hue and saturation, destination luminosity       |
| `LUMINOSITY`    | Source luminosity, destination hue and saturation       |

### Applicability

| Context       | `PASS_THROUGH` | All Others |
|---------------|----------------|------------|
| Node layer    | Yes            | Yes        |
| Paint (fill/stroke) | No      | Yes        |
| Effect        | No             | Yes        |

---

## 7. Opacity

### Node-Level Opacity

| Field     | Type     | Range   | Default | Description                              |
|-----------|----------|---------|---------|------------------------------------------|
| `opacity` | `number` | `0..1`  | `1.0`   | Overall node opacity (multiplies with all content) |

### Paint-Level Opacity

| Field     | Type     | Range   | Default | Description                              |
|-----------|----------|---------|---------|------------------------------------------|
| `opacity` | `number` | `0..1`  | `1.0`   | Individual fill/stroke paint opacity     |

### Color-Level Alpha

| Field | Type     | Range   | Description                                      |
|-------|----------|---------|--------------------------------------------------|
| `a`   | `number` | `0..1`  | Alpha channel within an RGBA color               |

**Effective opacity** = `node.opacity * paint.opacity * color.a`

All three levels multiply together. For example, a node with `opacity: 0.5` containing a
fill with `opacity: 0.8` and a color with `a: 0.9` yields a final alpha of `0.5 * 0.8 * 0.9 = 0.36`.

---

## 8. Clipping and Masks

### Frame Clipping

| Field          | Type      | Default | Description                                         |
|----------------|-----------|---------|-----------------------------------------------------|
| `clipsContent` | `boolean` | `true`  | Whether children outside frame bounds are clipped   |

Applies to: `FrameNode`, `ComponentNode`, `ComponentSetNode`, `InstanceNode`.
Does NOT apply to `GroupNode`.

### Mask Properties

| Field           | Type       | Default   | Description                                     |
|-----------------|------------|-----------|-------------------------------------------------|
| `isMask`        | `boolean`  | `false`   | Whether this node acts as a mask                |
| `maskType`      | `enum`     | `ALPHA`   | How the mask determines visibility              |
| `isMaskOutline` | `boolean`  | `false`   | REST API: whether mask uses outline only        |

**MaskType values:**

| Value       | Description                                                    |
|-------------|----------------------------------------------------------------|
| `ALPHA`     | Mask alpha channel determines pixel opacity of masked siblings |
| `VECTOR`    | Pixels inside fill regions are fully visible, outside hidden   |
| `LUMINANCE` | Mask luminance value determines pixel opacity                  |

**Mask behavior:** A mask node masks all of its **subsequent siblings** in the
parent's children array. Siblings listed before the mask are not affected.

---

## 9. Constraints and Layout

### 9.1 Fixed Layout Constraints

How a node is pinned relative to its parent when the parent resizes.

```
constraints: LayoutConstraint
```

**LayoutConstraint:**

| Field        | Type   | Description                                        |
|--------------|--------|----------------------------------------------------|
| `vertical`   | `enum` | Vertical constraint behavior                       |
| `horizontal` | `enum` | Horizontal constraint behavior                     |

**Vertical constraint values:**

| Value        | Description                                              |
|--------------|----------------------------------------------------------|
| `TOP`        | Fixed distance from top edge                             |
| `BOTTOM`     | Fixed distance from bottom edge                          |
| `CENTER`     | Centered vertically                                      |
| `TOP_BOTTOM` | Fixed distance from both top and bottom (stretches)      |
| `SCALE`      | Scales proportionally with parent                        |

**Horizontal constraint values:**

| Value        | Description                                              |
|--------------|----------------------------------------------------------|
| `LEFT`       | Fixed distance from left edge                            |
| `RIGHT`      | Fixed distance from right edge                           |
| `CENTER`     | Centered horizontally                                    |
| `LEFT_RIGHT` | Fixed distance from both left and right (stretches)      |
| `SCALE`      | Scales proportionally with parent                        |

### 9.2 Auto Layout (Flexbox-like)

These properties apply to frames with `layoutMode` set to `HORIZONTAL` or `VERTICAL`.

#### Frame-Level Auto Layout Properties

| Field                     | Type     | Default         | Description                                          |
|---------------------------|----------|-----------------|------------------------------------------------------|
| `layoutMode`              | `enum`   | `NONE`          | Auto-layout direction                                |
| `primaryAxisAlignItems`   | `enum`   | `MIN`           | Main-axis alignment of children                      |
| `counterAxisAlignItems`   | `enum`   | `MIN`           | Cross-axis alignment of children                     |
| `counterAxisAlignContent` | `enum`   | `AUTO`          | Cross-axis content distribution (wrap mode only)     |
| `primaryAxisSizingMode`   | `enum`   | `AUTO`          | How the frame sizes on the primary axis              |
| `counterAxisSizingMode`   | `enum`   | `AUTO`          | How the frame sizes on the counter axis              |
| `layoutWrap`              | `enum`   | `NO_WRAP`       | Whether children wrap to next line                   |
| `paddingTop`              | `number` | `0`             | Top padding in pixels                                |
| `paddingRight`            | `number` | `0`             | Right padding in pixels                              |
| `paddingBottom`           | `number` | `0`             | Bottom padding in pixels                             |
| `paddingLeft`             | `number` | `0`             | Left padding in pixels                               |
| `itemSpacing`             | `number` | `0`             | Gap between children on the primary axis             |
| `counterAxisSpacing`      | `number` | `0`             | Gap between wrapped rows/columns                     |
| `itemReverseZIndex`       | `boolean`| `false`         | Reverse z-order of children                          |
| `strokesIncludedInLayout` | `boolean`| `false`         | Whether strokes are included in layout calculations  |

**layoutMode values:**

| Value        | Description                          |
|--------------|--------------------------------------|
| `NONE`       | No auto-layout (manual positioning)  |
| `HORIZONTAL` | Children flow left-to-right          |
| `VERTICAL`   | Children flow top-to-bottom          |
| `GRID`       | Grid layout                          |

**primaryAxisAlignItems values:**

| Value           | Horizontal layout | Vertical layout |
|-----------------|-------------------|-----------------|
| `MIN`           | Left              | Top             |
| `CENTER`        | Center            | Center          |
| `MAX`           | Right             | Bottom          |
| `SPACE_BETWEEN` | Distributed       | Distributed     |

**counterAxisAlignItems values:**

| Value      | Horizontal layout | Vertical layout |
|------------|-------------------|-----------------|
| `MIN`      | Top               | Left            |
| `CENTER`   | Center            | Center          |
| `MAX`      | Bottom            | Right           |
| `BASELINE` | Text baseline     | Text baseline   |

**counterAxisAlignContent values (only meaningful when `layoutWrap` is `WRAP`):**

| Value           | Description                              |
|-----------------|------------------------------------------|
| `AUTO`          | Default alignment behavior               |
| `SPACE_BETWEEN` | Distribute wrapped rows/columns evenly   |

**primaryAxisSizingMode / counterAxisSizingMode values:**

| Value   | Description                                       |
|---------|---------------------------------------------------|
| `FIXED` | Frame size is user-defined (does not auto-resize) |
| `AUTO`  | Frame resizes to fit its children                 |

**layoutWrap values:**

| Value     | Description                                    |
|-----------|------------------------------------------------|
| `NO_WRAP` | Children overflow (single row/column)          |
| `WRAP`    | Children wrap to the next row/column           |

#### Child-Level Auto Layout Properties

| Field              | Type     | Default    | Description                                      |
|--------------------|----------|------------|--------------------------------------------------|
| `layoutAlign`      | `enum`   | `INHERIT`  | How this child aligns on the counter axis         |
| `layoutGrow`       | `number` | `0`        | Whether this child stretches on the primary axis  |
| `layoutPositioning`| `enum`   | `AUTO`     | Whether this child participates in auto-layout    |

**layoutAlign values:**

| Value     | Description                                          |
|-----------|------------------------------------------------------|
| `INHERIT` | Use the parent's `counterAxisAlignItems` setting     |
| `STRETCH` | Stretch to fill the parent's counter axis            |
| `MIN`     | Align to the start of the counter axis               |
| `CENTER`  | Center on the counter axis                           |
| `MAX`     | Align to the end of the counter axis                 |

**layoutGrow values:**

| Value | Description                                           |
|-------|-------------------------------------------------------|
| `0`   | Fixed size (does not grow)                            |
| `1`   | Fills remaining space on primary axis (like flex: 1)  |

**layoutPositioning values:**

| Value      | Description                                        |
|------------|----------------------------------------------------|
| `AUTO`     | Participates in auto-layout flow                   |
| `ABSOLUTE` | Absolutely positioned (ignores auto-layout flow)   |

### 9.3 Bounding Boxes

| Field                  | Type        | Description                                         |
|------------------------|-------------|-----------------------------------------------------|
| `absoluteBoundingBox`  | `Rectangle` | Bounding box in absolute (page) coordinates         |
| `absoluteRenderBounds` | `Rectangle` | Actual rendered bounds (includes strokes, shadows)  |
| `relativeTransform`    | `Transform` | 2x3 transform matrix relative to parent             |
| `size`                 | `Vector`    | Width and height of the node `{ x: w, y: h }`       |
| `preserveRatio`        | `boolean`   | Whether aspect ratio is locked                       |

**Rectangle:**

| Field    | Type     | Description     |
|----------|----------|-----------------|
| `x`      | `number` | X position      |
| `y`      | `number` | Y position      |
| `width`  | `number` | Width in pixels  |
| `height` | `number` | Height in pixels |

---

## 10. Text Properties

### 10.1 TypeStyle (REST API)

The REST API returns text properties bundled into a `TypeStyle` object on TEXT nodes.

| Field                        | Type         | Default      | Description                                 |
|------------------------------|--------------|--------------|---------------------------------------------|
| `fontFamily`                 | `string`     |              | Font family name (e.g., `"Inter"`)          |
| `fontPostScriptName`         | `string`     |              | PostScript font name                        |
| `fontStyle`                  | `string`     |              | Visual weight/emphasis (e.g., `"Bold"`, `"Italic"`) |
| `fontWeight`                 | `number`     | `400`        | Numeric weight (100-900)                    |
| `fontSize`                   | `number`     |              | Font size in pixels                         |
| `italic`                     | `boolean`    | `false`      | Whether text is italicized                  |
| `textAlignHorizontal`        | `enum`       | `LEFT`       | Horizontal alignment                        |
| `textAlignVertical`          | `enum`       | `TOP`        | Vertical alignment                          |
| `letterSpacing`              | `number`     | `0`          | Letter spacing in pixels                    |
| `lineHeightPx`               | `number`     |              | Resolved line height in pixels              |
| `lineHeightPercent`          | `number`     | `100`        | Line height as % of font size (deprecated)  |
| `lineHeightPercentFontSize`  | `number`     |              | Line height as % of font size               |
| `lineHeightUnit`             | `enum`       |              | Unit for line height value                  |
| `textCase`                   | `enum`       | `ORIGINAL`   | Text case transformation                    |
| `textDecoration`             | `enum`       | `NONE`       | Text decoration                             |
| `textAutoResize`             | `enum`       | `NONE`       | How the text box auto-resizes               |
| `textTruncation`             | `enum`       | `DISABLED`   | Truncation behavior                         |
| `maxLines`                   | `number|null`| `null`       | Max lines before truncation                 |
| `paragraphSpacing`           | `number`     | `0`          | Space between paragraphs in pixels          |
| `paragraphIndent`            | `number`     | `0`          | First-line indentation in pixels            |
| `listSpacing`                | `number`     | `0`          | Space between list items in pixels          |
| `fills`                      | `Paint[]`    |              | Paints applied to the text characters       |
| `hyperlink`                  | `Hyperlink`  |              | URL or frame link                           |
| `opentypeFlags`              | `object`     | `{}`         | OpenType feature flags `{ tag: value }`     |
| `semanticWeight`             | `enum`       |              | `BOLD` or `NORMAL` semantic override        |
| `semanticItalic`             | `enum`       |              | `ITALIC` or `NORMAL` semantic override      |
| `isOverrideOverTextStyle`    | `boolean`    |              | Whether style overrides exist               |

### 10.2 Plugin API Text Properties

In the Plugin API, text properties are accessed as individual properties on `TextNode`:

| Property                    | Type                          | Description                              |
|-----------------------------|-------------------------------|------------------------------------------|
| `fontName`                  | `FontName \| figma.mixed`     | `{ family: string, style: string }`      |
| `fontSize`                  | `number \| figma.mixed`       | Font size in pixels (min: 1)             |
| `fontWeight`                | `number` (readonly)           | Numeric weight (400=Regular, 700=Bold)   |
| `textCase`                  | `TextCase \| figma.mixed`     | Case transformation                      |
| `textDecoration`            | `TextDecoration \| figma.mixed`| Decoration style                        |
| `textDecorationStyle`       | `TextDecorationStyle \| figma.mixed \| null` | Decoration line style |
| `textDecorationOffset`      | `TextDecorationOffset \| figma.mixed \| null`| Decoration offset    |
| `textDecorationThickness`   | `TextDecorationThickness \| figma.mixed \| null`| Decoration weight |
| `textDecorationColor`       | `TextDecorationColor \| figma.mixed \| null`  | Decoration color    |
| `textDecorationSkipInk`     | `boolean \| figma.mixed \| null`| Skip descenders    |
| `letterSpacing`             | `LetterSpacing \| figma.mixed`| Character spacing                        |
| `lineHeight`                | `LineHeight \| figma.mixed`   | Line spacing                             |
| `paragraphSpacing`          | `number`                      | Paragraph gap in pixels                  |
| `paragraphIndent`           | `number`                      | First-line indent in pixels              |
| `listSpacing`               | `number`                      | List item gap in pixels                  |
| `textAlignHorizontal`       | `enum`                        | Horizontal alignment                     |
| `textAlignVertical`         | `enum`                        | Vertical alignment                       |
| `textAutoResize`            | `enum`                        | Resize mode                              |
| `textTruncation`            | `enum`                        | Truncation behavior                      |
| `maxLines`                  | `number \| null`              | Max lines                                |
| `hangingPunctuation`        | `boolean`                     | Hang quotes outside bounds               |
| `hangingList`               | `boolean`                     | Hang list markers outside bounds         |
| `leadingTrim`               | `LeadingTrim \| figma.mixed`  | Trim vertical space above/below glyphs   |
| `hyperlink`                 | `HyperlinkTarget \| null \| figma.mixed` | URL or node link      |
| `openTypeFeatures`          | `object` (readonly)           | Enabled/disabled OT features             |
| `autoRename`                | `boolean`                     | Auto-derive node name from text content  |

### 10.3 Text Enum Values

**TextAlignHorizontal:**

| Value       | Description        |
|-------------|--------------------|
| `LEFT`      | Left-aligned       |
| `CENTER`    | Center-aligned     |
| `RIGHT`     | Right-aligned      |
| `JUSTIFIED` | Justified          |

**TextAlignVertical:**

| Value    | Description      |
|----------|------------------|
| `TOP`    | Top-aligned      |
| `CENTER` | Center-aligned   |
| `BOTTOM` | Bottom-aligned   |

**TextCase:**

| Value              | Description                          |
|--------------------|--------------------------------------|
| `ORIGINAL`         | No transformation                    |
| `UPPER`            | ALL UPPERCASE                        |
| `LOWER`            | all lowercase                        |
| `TITLE`            | Title Case                           |
| `SMALL_CAPS`       | Small Capitals                       |
| `SMALL_CAPS_FORCED`| Forced Small Capitals (all letters)  |

**TextDecoration:**

| Value           | Description                |
|-----------------|----------------------------|
| `NONE`          | No decoration              |
| `UNDERLINE`     | Underlined text            |
| `STRIKETHROUGH` | Strikethrough text         |

**TextAutoResize:**

| Value              | Description                                        |
|--------------------|----------------------------------------------------|
| `NONE`             | Text box is fixed size                             |
| `HEIGHT`           | Text box height auto-resizes, width is fixed       |
| `WIDTH_AND_HEIGHT`| Both dimensions auto-resize to fit content         |
| `TRUNCATE`         | Fixed size, text truncated with ellipsis           |

**LineHeightUnit (REST API):**

| Value            | Description                          |
|------------------|--------------------------------------|
| `PIXELS`         | Line height in absolute pixels       |
| `FONT_SIZE_%`    | Line height as % of font size        |
| `INTRINSIC_%`    | Line height as % of intrinsic height |

**LetterSpacing (Plugin API composite type):**

```
{ value: number, unit: "PIXELS" | "PERCENT" }
```

**LineHeight (Plugin API composite type):**

```
{ value: number, unit: "PIXELS" | "PERCENT" }
| { unit: "AUTO" }
```

---

## 11. Vector / Path Properties

### 11.1 Path (REST API) / VectorPath (Plugin API)

Fill and stroke geometry is represented as arrays of path objects.

| Field         | Type     | Description                                     |
|---------------|----------|-------------------------------------------------|
| `fillGeometry`  | `Path[]` | Paths defining the filled area of the shape   |
| `strokeGeometry`| `Path[]` | Paths defining the stroked outline            |

**Path (REST API):**

| Field        | Type     | Description                                       |
|--------------|----------|---------------------------------------------------|
| `path`       | `string` | SVG path data string (M, L, C, Q, Z commands)     |
| `windingRule`| `enum`   | Fill rule for determining inside/outside           |

**VectorPath (Plugin API):**

| Field        | Type                  | Description                                |
|--------------|-----------------------|--------------------------------------------|
| `data`       | `string` (readonly)   | SVG path data string                       |
| `windingRule`| `WindingRule \| "NONE"`| Fill rule or NONE for open paths           |

**WindingRule values:**

| Value     | Description                                                   |
|-----------|---------------------------------------------------------------|
| `NONZERO` | Non-zero winding rule (standard SVG default)                 |
| `EVENODD` | Even-odd rule (alternating inside/outside on intersections)  |

**SVG Path Commands used in `data`/`path`:**

| Command | Parameters      | Description                           |
|---------|-----------------|---------------------------------------|
| `M`     | `x y`           | Move to (start new subpath)           |
| `L`     | `x y`           | Line to                               |
| `C`     | `x1 y1 x2 y2 x y` | Cubic bezier curve to              |
| `Q`     | `x1 y1 x y`    | Quadratic bezier curve to             |
| `Z`     |                 | Close path (line back to start)       |

### 11.2 VectorNetwork (Plugin API - Advanced)

A graph-based representation of vector geometry (superset of paths).

**VectorNetwork:**

| Field      | Type               | Description                              |
|------------|--------------------|------------------------------------------|
| `vertices` | `VectorVertex[]`   | Array of points in the graph             |
| `segments` | `VectorSegment[]`  | Array of edges connecting vertices       |
| `regions`  | `VectorRegion[]`   | Array of filled regions (optional)       |

**VectorVertex:**

| Field            | Type            | Default       | Description                       |
|------------------|-----------------|---------------|-----------------------------------|
| `x`              | `number`        |               | X coordinate                      |
| `y`              | `number`        |               | Y coordinate                      |
| `strokeCap`      | `StrokeCap`     | (from node)   | End cap at this vertex            |
| `strokeJoin`     | `StrokeJoin`    | (from node)   | Join style at this vertex         |
| `cornerRadius`   | `number`        | (from node)   | Corner radius at this vertex      |
| `handleMirroring`| `HandleMirroring`| (from node)  | Bezier handle behavior            |

**VectorSegment:**

| Field         | Type     | Default           | Description                          |
|---------------|----------|--------------------|--------------------------------------|
| `start`       | `number` |                    | Index into vertices array (start)    |
| `end`         | `number` |                    | Index into vertices array (end)      |
| `tangentStart`| `Vector` | `{ x: 0, y: 0 }`  | Cubic bezier control point at start  |
| `tangentEnd`  | `Vector` | `{ x: 0, y: 0 }`  | Cubic bezier control point at end    |

When both `tangentStart` and `tangentEnd` are `{0,0}`, the segment is a straight line.

**VectorRegion:**

| Field        | Type          | Description                                  |
|--------------|---------------|----------------------------------------------|
| `windingRule` | `WindingRule` | Fill rule for this region                    |
| `loops`      | `number[][]`  | Arrays of segment indices forming closed loops|
| `fills`      | `Paint[]`     | Region-specific fill paints (optional)       |
| `fillStyleId`| `string`      | Fill style reference (optional)              |

**HandleMirroring values:**

| Value      | Description                                            |
|------------|--------------------------------------------------------|
| `NONE`     | Handles move independently                             |
| `ANGLE`    | Handles maintain same angle but can differ in length   |
| `ANGLE_AND_LENGTH` | Handles mirror both angle and length          |

---

## 12. Image Fill Properties

Detailed reference for image fills (subset of Paint, expanded here for clarity).

### ScaleMode Behavior

| ScaleMode | `imageTransform` | `scalingFactor` | Behavior                           |
|-----------|-------------------|-----------------|------------------------------------|
| `FILL`    | Ignored           | Ignored         | Scales to cover entire bounds, crops excess |
| `FIT`     | Ignored           | Ignored         | Scales to fit within bounds, letterboxes    |
| `CROP`    | **Used**          | Ignored         | 2x3 transform positions/scales the image   |
| `TILE`    | Ignored           | **Used**        | Repeats image, `scalingFactor` controls tile size |
| `STRETCH` | Ignored           | Ignored         | REST API only: stretches to match bounds exactly |

### imageTransform (for CROP mode)

A 2x3 affine transformation matrix (`Transform` = `number[2][3]`) that maps from
image coordinates to the layer's coordinate space:

```
[[a, b, tx],
 [c, d, ty]]
```

Where `a,d` = scale, `b,c` = skew/rotation, `tx,ty` = translation.

The transform defines which portion of the image is visible and how it is
positioned within the layer bounds.

### Image Reference

| API         | Field       | Description                                   |
|-------------|-------------|-----------------------------------------------|
| REST API    | `imageRef`  | Hash reference to retrieve via Images endpoint|
| Plugin API  | `imageHash` | Hash for `figma.getImageByHash()`             |

---

## 13. Transform

Used for gradient transforms, image transforms, and node positioning.

**Transform** = `number[2][3]` (2x3 affine matrix)

```
[[a, c, tx],
 [b, d, ty]]
```

| Element | Description                  |
|---------|------------------------------|
| `a`     | X scale                      |
| `b`     | Y skew                       |
| `c`     | X skew                       |
| `d`     | Y scale                      |
| `tx`    | X translation                |
| `ty`    | Y translation                |

Identity transform: `[[1, 0, 0], [0, 1, 0]]`

Used in:
- `relativeTransform` on nodes (position relative to parent)
- `gradientTransform` on gradient paints
- `imageTransform` on image paints (CROP mode)

---

## Summary: Property Hierarchy

```
Node
  |-- opacity: number
  |-- blendMode: BlendMode
  |-- clipsContent: boolean (frames)
  |-- isMask / maskType (masks)
  |-- constraints: LayoutConstraint
  |-- Auto-layout properties (frames)
  |-- cornerRadius / rectangleCornerRadii
  |-- cornerSmoothing
  |-- effects: Effect[]
  |     |-- DropShadowEffect
  |     |-- InnerShadowEffect
  |     |-- BlurEffect (LAYER_BLUR / BACKGROUND_BLUR)
  |
  |-- fills: Paint[]
  |     |-- SolidPaint  { color, opacity, blendMode }
  |     |-- GradientPaint  { gradientStops, gradientTransform, type }
  |     |-- ImagePaint  { scaleMode, imageHash, imageTransform, filters }
  |     |-- VideoPaint  { videoHash }
  |     |-- PatternPaint  { sourceNodeId, tileType, scalingFactor }
  |
  |-- strokes: Paint[]
  |-- strokeWeight / individualStrokeWeights
  |-- strokeAlign / strokeCap / strokeJoin / strokeDashes
  |
  |-- fillGeometry: Path[]   (vector nodes)
  |-- strokeGeometry: Path[]
  |
  |-- TypeStyle (text nodes)
        |-- fontFamily, fontSize, fontWeight, fontStyle
        |-- textAlignHorizontal, textAlignVertical
        |-- letterSpacing, lineHeight, paragraphSpacing
        |-- textCase, textDecoration
        |-- textAutoResize, textTruncation, maxLines
        |-- fills (text-specific fills)
```
