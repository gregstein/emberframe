# Emberframe user guide

Hold Shift while drawing to constrain shapes and lines. Hold Space to pan. Ctrl+scroll zooms. Press Enter to finish a Pen path, or click its first point to close it.

## Editing images

Use File > Open (Ctrl+O) to edit an image, or File > Import (Ctrl+R) to add one to the current document.

Brush, Pencil and Eraser change the pixels of the bitmap under the pointer. Off a bitmap, Brush and Pencil draw vector strokes. Brush paints in the stroke color, and Paint Bucket fills in the fill color. Set Brush size in the Properties panel while the Brush or Eraser is active.

Drag with Crop, then press Enter or double-click to apply. Esc cancels.

Save never overwrites the opened image. It creates an `.ember` document that keeps your objects, and File > Export writes a flat image.

Marquee (M), Lasso (L) and Magic Wand (W) select pixels on a bitmap. Hold Shift to add to the selection, or Alt to subtract. Painting, filling and filters stay inside the selection. Cut, Copy and Delete act on the selected pixels, and Paste adds them as a new bitmap. Ctrl+D or Esc drops the selection.

Rubber Stamp (S): Alt+click to set the source, then drag to clone from it.

## Reshaping vectors

Scale (Q) resizes everything selected from its handles. Corners keep proportions unless you hold Shift.

Freeform (O) pulls a path or shape near the pointer, and Knife (Y, press again from Pencil) cuts selected vectors along a dragged line. With nothing selected, Knife cuts every vector it crosses. Both turn rectangles and ellipses into paths.

## Web areas

Hotspot (J) and Slice (K) draw areas over the artwork. Move and resize them with the Pointer, and set a hotspot's link in Properties. Press 2 to hide or show them.

They never appear in the flattened image. Exporting `name.png` also writes `name_slice1.png`, `name_slice2.png` and so on for slices, and `name_map.html` with an image map for hotspots.

## Panels

The Window menu opens and closes every panel, and F4 hides or restores them all. Click a tab to bring it forward, and double-click the front tab to collapse its group.

- **States**: each state has its own objects. Play the animation, turn on onion skinning, and share a layer across states from the Layers panel. Double-click a page, state, layer or object to rename it.
- **Optimize**: pick GIF, Animated GIF, JPEG, PNG 8, PNG 24 or PNG 32. The panel estimates the file size, and Export Page uses these settings.
- **Styles**: click a style to apply it, or to set the colors for the next shape. Styles hold fill, stroke and opacity.
- **Path**: Union, Intersect, Punch, Crop, Join, Split, Simplify, Expand Stroke, Inset and Outset.
- **Auto Shapes**: click a tile to insert it, then adjust it in Auto Shape Properties.
- **Document Library**: New makes a symbol from the selection. Insert places it, Update redefines it from the selection, and Delete removes it.
- **Common Library**: click an item to add it to the document as a symbol.
- **Find**: replace text or colors on this page or on all pages.
- **Color Palette**: click a color to fill with it, or Shift+click for the stroke. It also suggests complementary, analogous and triad colors for the current fill.
- **Symbol Properties**: rename the selected symbol, see how many copies exist, and break it apart.

The panel menu at the right of each tab row collapses or closes that group. Expanded Mode in the top bar switches between open panels and tabs only.

## Comparing exports

Preview shows the page as it will export. 2-Up puts the original next to it, and 4-Up compares the original with three export settings and their file sizes. Drag to pan in these views.
