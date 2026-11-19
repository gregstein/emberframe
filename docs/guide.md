# Emberframe user guide

Hold Shift while drawing to constrain shapes and lines. Hold Space to pan. Ctrl+scroll zooms. Press Enter to finish a Pen path, or click its first point to close it.

## Curves

With the Pen (P), click to place a corner. Press and drag instead to place a smooth point: the two handles you pull out bend the path on both sides of it.

With Subselection (A), select a path to see its points. Click a point to pick it, which shows its handles; Shift+click picks more, and dragging a box over empty canvas picks every point inside it. Drag a picked point to move it, with any others you picked, or drag a handle to reshape the curve. The opposite handle turns with it so the point stays smooth; hold Alt to move one handle on its own and make a cusp. Alt+drag a corner pulls handles out of it, and Alt+click a point turns it back into a corner. Delete removes the picked points.

The Pen also edits a selected path. Click a segment to add a point there. Click a curve point to make it a corner, and click a corner to remove it. Drag from a point to pull handles out of it. Click the loose end of an open path to carry on drawing from it, and finish on the loose end of another open path to join the two.

Ellipses turn into paths of a few curve points when you edit their points. A rectangle asks first, because as a path it no longer has a roundness setting. Freeform, Knife and the path operations give curved paths back: a circle joined to another is still made of curves.

## Editing images

Use File > Open (Ctrl+O) to edit an image, or File > Import (Ctrl+R) to add one to the current document.

Brush, Pencil and Eraser change the pixels of the bitmap under the pointer. Off a bitmap, Brush and Pencil draw vector strokes. Brush paints in the stroke color, and Paint Bucket fills in the fill color. Set Brush size in the Properties panel while the Brush or Eraser is active.

Drag with Crop, then press Enter or double-click to apply. Esc cancels.

Save never overwrites the opened image. It creates a `.pisi` document that keeps your objects, and File > Export writes a flat image.

Marquee (M), Lasso (L) and Magic Wand (W) select pixels on a bitmap. Hold Shift to add to the selection, or Alt to subtract. Painting, filling and filters stay inside the selection. Cut, Copy and Delete act on the selected pixels, and Paste adds them as a new bitmap. Drag inside the selection with the Pointer, or press the arrow keys, to move the selected pixels; hold Alt to leave a copy behind. Dragging inside it with Marquee or Lasso moves only the selection border, and with Ctrl held it moves the pixels. Ctrl+D or Esc drops the selection.

The Select menu changes a pixel selection. Select Similar adds every pixel of a like color anywhere on the bitmap, and Select Inverse (Ctrl+Shift+I) selects everything else. Expand Marquee and Contract Marquee grow or shrink it by a number of pixels, Border Marquee turns it into a band along its edge, and Smooth Marquee evens out a ragged edge. Feather fades the edge of the selection out over a radius, so that deleting, copying, moving, painting and filters fade with it. With no pixels selected, Select Inverse selects the objects that were not selected.

Rubber Stamp (S): Alt+click to set the source, then drag to clone from it.

Modify > Flatten Selection (Ctrl+Alt+Shift+Z) turns the selected objects into one bitmap. Merge Down (Ctrl+E) flattens them into the bitmap right under them. Flatten Layers makes one bitmap of everything on the layers that show, on one layer; hidden layers are removed, and slices and hotspots are kept. Modify > Canvas > Trim Canvas cuts the canvas back to the artwork.

Edit > Insert > Empty Bitmap adds a clear bitmap the size of the canvas to paint on. Edit > Clone (Ctrl+Shift+D) copies the selection in place, and Paste Attributes (Ctrl+Alt+Shift+V) gives the selection the fill, stroke, opacity, blend mode and filters of what was copied. Filters > Repeat Filter (Ctrl+Alt+Shift+F) applies the last filter again. File > Revert goes back to the file as it was last saved. View > Fit Selection (Ctrl+Alt+0) zooms in on the selection.

## Reshaping vectors

Scale (Q) resizes everything selected from its handles. Corners keep proportions unless you hold Shift.

Skew and Distort share the Scale button: hold the button down to list them, or press Q again to step through the three. With any of them, drag outside the selection box to rotate; hold Shift to turn in steps of 15 degrees. Rotation turns around the small circle in the middle of the box, which you can drag somewhere else first.

- **Skew**: drag a side handle to slant the selection, or a corner handle for perspective, where the corner and its neighbour move apart or together.
- **Distort**: drag a corner handle to move that corner alone, or a side handle to move the whole side.

While you drag, the artwork only previews the transform, and you can go on with another handle or another of the three tools. Press Enter or double-click inside the box to apply it, or Esc to put everything back. Dragging inside the box moves the selection.

Modify > Transform lists the same tools, with Rotate 180, Rotate 90 and the two flips. Numeric Transform (Ctrl+Shift+T) scales by a percentage, resizes to a size in pixels or rotates by an angle. It works around the center point if you have dragged it, and around the middle of the selection otherwise.

Text stays text when it is transformed, so you can still edit it and change its size. A symbol stays linked to the library, a rectangle keeps its roundness and an auto shape its settings. Modify > Transform > Remove Transformations stands any of them upright again. Flip Horizontal and Flip Vertical mirror text, symbols and groups too. Bitmaps are redrawn. Web areas are not transformed. Text > Convert to Paths turns text into the outlines of its letters.

Freeform (O) pulls a path or shape near the pointer, and Knife (Y, press again from Pencil) cuts selected vectors along a dragged line. With nothing selected, Knife cuts every vector it crosses. Both turn rectangles and ellipses into paths.

## Live filters

Filters in Properties adds effects to the selected objects: color adjustments, bevels and embosses, blurs, noise, shadows and glows, and sharpening. Pick one from the list to add it and set it. Each filter is listed with a box that switches it off, its name, which opens its settings again, and a minus that removes it. Filters are applied from the top of the list down, every time the object is drawn, so the object itself can still be edited. Drag a filter's name up or down to change its place. A style made from an object keeps its filters.

Levels shows how many pixels have each level, with the levels taken as black, mid gray and white on the way in and as black and white on the way out. Curves shows a curve from input level to output level: click it to add a point, drag a point to bend it, and drag a point out of the box to remove it. Both have a Channel list, with settings for red, green and blue apart from the ones all three share, and an Auto button that stretches each color from its darkest level to its lightest. In Levels the triangles under the histogram and under the ramp can be dragged instead of typing the numbers. The three eyedroppers set the black, gray or white point: pick one, then click a color on the object.

Filter dialogs have a Preview box. With it off, the object shows as it was before the dialog opened, until the box is ticked again or the dialog is closed with OK.

Photoshop Live Effects is one filter holding Drop Shadow, Inner Shadow, Outer Glow, Inner Glow, Bevel and Emboss, Satin, Color Overlay, Gradient Overlay, Pattern Overlay and Stroke. Tick the ones you want, and click a name to see its settings: blend mode, opacity, color, angle, distance, spread or choke, size and noise for shadows and glows; style, technique, depth, direction, soften, altitude, highlight and shadow for a bevel; style, angle and scale for a gradient; position and fill type for a stroke. Patterns come from a short built-in list.

A gradient is shown with its stops under it. Click under the gradient to add a stop, drag a stop to move it, and pull one away from the gradient to remove it. The stop that is picked shows its color, its place in percent and its opacity. Presets replaces the gradient with a ready-made one. The overlay, a gradient stroke and a gradient glow each have a gradient of their own; a gradient glow starts at the edge of the object, and Jitter scatters its colors.

The drop shadow and the outer glow are mixed into what lies under the object by their own blend modes. A filter that comes after Photoshop Live Effects in the list works on them as it does on the object.

## Masks

A mask hides part of an object. Modify > Mask has the commands:

- **Group as Mask**: select two or more objects; the one on top becomes the mask of the others. The lighter the mask is, the more shows through it, so use a white shape to cut out a clean piece.
- **Paste as Mask**: copy an object, select another and choose this; the copied object masks it by its outline. **Paste Inside** (Ctrl+Shift+V) does the opposite: what you copied appears inside the selected object, which stays in view around it.
- **Reveal All** and **Hide All** add a bitmap mask that shows everything or nothing, to paint on. With pixels selected on a bitmap, **Reveal Selection** and **Hide Selection** make the mask from the selection.
- **Disable Mask** switches the mask off without removing it, and **Delete Mask** removes it, after asking whether to apply it to the object first.

A masked object shows Mask in Properties: Path Outline or Grayscale Appearance for a vector mask, with Show fill and stroke, and Grayscale Appearance or Alpha Channel for a bitmap mask. Edit Mask there, or a click on the mask's thumbnail in Layers, selects the mask itself so you can move, reshape or paint it with any tool. White paint reveals and black hides. Select something else to finish.

In the Layers panel a chain between an object and its mask shows that the two are linked and move together. Click it to let the mask go, so the object can be moved under it, and again to link them. Paste as Mask asks whether a vector is to become a bitmap mask or stay a vector, and whether to replace a mask that is already there or add to it.

## Tool groups

A button with a small triangle stands for several tools. Hold it down to list them, or press its key again to walk through them; the button shows the one used last.

- Pointer (V): Select Behind picks the next object down under the mouse with each click.
- Scale (Q): after Skew and Distort comes 9-Slice Scaling. Four blue lines cut the selection into nine parts; drag a line to move it, then drag a handle of the selection: the corners keep their size and only the middle stretches, so rounded corners stay round.
- Crop (C): Export Area exports only the part of the page you drag over. Press Enter or double-click, and Image Preview opens for that part.
- Marquee (M): Oval Marquee selects an oval. Lasso (L): Polygon Lasso selects with straight sides; click each corner, and double-click or click the first corner to close it.
- Blur (R): Sharpen, Dodge (lightens), Burn (darkens) and Smudge (drags paint along the stroke) work under the brush like Blur. Dodge and Burn have an Exposure and a Range of tones in Properties; Smudge has a Pressure and can start loaded with the stroke color.
- Rubber Stamp (S): Replace Color changes the From color into the To color, within its Tolerance; with no From color it changes the color the stroke starts on. Red Eye Removal takes the red out of eyes in the area you drag over, by its Tolerance and Strength.
- Pen (P): Vector Path draws a path freehand. Redraw Path replaces part of a path: start the stroke on the path and end it on the path again.
- Freeform (O): Reshape Area pushes every point within its circle along with the mouse. The two Path Scrubbers make a stroke heavier or lighter where you rub over it, at the Rate set in Properties; a path treated this way becomes many short straight pieces.
- Hotspot (J): Circle Hotspot is drawn from its middle outward, and Polygon Hotspot corner by corner like the Polygon Lasso. Slice (K): Polygon Slice is drawn the same way. In the exported image map they are written as circles and polygons.
- Rectangle (U): Ellipse, and Polygon, which is drawn from its middle outward; Properties sets its number of sides, or makes it a star.
- Paint Bucket (G): Gradient fills the selected object with a gradient from where you start dragging to where you let go.

## Gradient and pattern fills

With a vector object selected, the list beside its fill in Properties chooses what it is filled with: Solid for one color, one of twelve gradients (Linear, Radial, Ellipse, Rectangle, Cone, Contour, Satin, Starburst, Folds, Bars, Ripples, Waves), or Pattern. A new gradient runs from the object's fill color to white.

With the Pointer, a gradient shows two handles on the canvas: a round one where it starts and a square one where it ends. Drag them to move, turn, lengthen or shorten the gradient; they may lie outside the object. Contour follows the outline of the object and has no handles.

An Ellipse or Rectangle gradient has a third, smaller handle that sets how wide it is. Web Dither fills with two colors pixel by pixel in turn. Text can be filled with a gradient or pattern like any shape.

Under the fill list, Edge sets how the edge of the fill is drawn: Hard, Anti-Alias, or Feather with the number of pixels it fades over. Texture lays one of the patterns into the fill, as strongly as its percentage.

Edit opens the gradient's colors: click under the gradient to add a color stop, drag a stop to move it, pull one away to remove it, and set the color and place of the stop that is picked. Opacity stops work the same way above the gradient: each sets how opaque the gradient is at its place. Presets replaces the gradient with a ready-made one. For a pattern, Edit chooses the pattern and its scale. Styles, Paste Attributes and files keep the fill.

## Image Preview

File > Image Preview (Ctrl+Shift+X) shows the export settings beside the image they give, with its format, file size and dimensions, and the image changes as you change the settings. Export saves it; OK keeps the settings without exporting, and Cancel puts them back.

## Printing

File > Page Setup sets the paper, which way it is turned, the margins, and whether the page is scaled to fit the paper. File > Print (Ctrl+P) sends the page to a printer set up on the computer, or saves it as a PDF file. The page prints as it exports, without slices and hotspots, at 72 pixels to the inch; a page larger than the paper is printed on several sheets unless Fit to page is on. Print range chooses the page on show, all pages of the document, or a range of them. Printing needs the system's print service (CUPS).

## Groups

Modify > Group (Ctrl+G) makes the selected objects move and scale as one. To change one member without ungrouping, click it with the Subselection tool, or choose Select > Subselect to select all the members of the selected group. A member can then be moved, recolored, reshaped or deleted on its own. The group is whole again as soon as none of its members is selected, and Select > Superselect selects the whole group from a member.

## Text

With a text object selected, Properties shows its font and size, buttons for bold, italic and underline, and buttons that align its lines left, centered or right. The same is in the Text menu: Font lists the fonts installed on the computer, Size has Smaller (Ctrl+Shift+,) and Larger (Ctrl+Shift+.) and a list of sizes, Style has Plain, Bold (Ctrl+B), Italic (Ctrl+I) and Underline (Ctrl+U), and Align has Left, Centered Horizontally, Right, Justified (every line but the last is spread from edge to edge by its spaces) and Stretched (every line is widened to fill the box). Set with nothing selected, they apply to the next text you type. For a font with no bold or italic face of its own, bold is made by thickening the letters and italic by leaning them.

Text > Attach to Path (Ctrl+Shift+Y) sets the selected text along the selected path or shape, which then belongs to the text and moves with it. Around a closed shape the text starts at the left and runs over the top. Properties shows how its letters stand (Text > Orientation: Rotate Around Path, Vertical, Skew Vertical, Skew Horizontal) and a Text offset that starts it further along. Reverse Direction runs it the other way, and Detach from Path gives the text and the path back.

Text > Check Spelling (Shift+F7) goes through the selected texts, or every text on the page, and stops at each word the dictionary does not have. Change puts the word in the field in its place, and Change All does so everywhere; Ignore passes over this one place and Ignore All over the word; Add to Personal makes the word right from now on. It checks in the language of the interface, with the spelling dictionaries installed on the system (the `enchant-2` or `hunspell` program and a dictionary such as `hunspell-tr`).

## Guides

Drag out of the ruler along the top or down the side to put a guide on the page. With the Pointer, drag a guide to move it, and drag it back onto a ruler to remove it. Objects being moved, drawn or resized snap to guides. Double-click a guide to type its place. The Pen and points dragged with the Subselection tool snap to guides too. View > Guides shows or hides them (Ctrl+;), locks them, turns snapping off, and clears them all; Edit Guides there also sets their color.

Smart guides appear while you move an object: a line shows where one of its edges or its middle lines up with another object or with the page, and the object snaps there. View > Smart Guides turns the lines or the snapping off. View > Slice Guides shows or hides the red lines along the edges of slices.

## Web areas

Hotspot (J) and Slice (K) draw areas over the artwork. Edit > Insert > Hotspot (Ctrl+Shift+U) and Rectangular Slice put one over the selected object; with several objects selected, you are asked whether to make one for all of them or one each. Move and resize them with the Pointer, and set a hotspot's link in Properties. Press 2 to hide or show them.

They never appear in the flattened image. Exporting `name.png` also writes `name_slice1.png`, `name_slice2.png` and so on for slices, and `name_map.html` with an image map for hotspots.

## Panels

The Window menu opens and closes every panel, and F4 hides or restores them all. Click a tab to bring it forward, and double-click the front tab to collapse its group.

- **Layers**: the list at the top sets the blend mode of the selected objects, which is how their colors mix with the artwork under them, and Opacity how much of it shows through. The same list is in Properties. A page that uses blend modes is shown as it will export, pixel for pixel.
- **States**: each state has its own objects. Play the animation, turn on onion skinning, and share a layer across states from the Layers panel. Double-click a page, state, layer or object to rename it.
- **Optimize**: pick GIF, Animated GIF, JPEG, PNG 8, PNG 24 or PNG 32. The panel estimates the file size, and Export Page uses these settings. For a JPEG, Smoothing (0 to 8) blurs the picture a little to make the file smaller. Selective quality keeps part of the page at a quality of its own: select the pixels, choose Modify > Selective JPEG > Save Selection as JPEG Mask, and tick Selective quality. The mask shows as a pink tint. Settings can also keep all text and all buttons at the selective quality, and change the color of the tint. A symbol is a button when its Type in Symbol Properties says so, as the buttons of the Common Library do.
- **Styles**: click a style to apply it, or to set the colors for the next shape. Styles hold fill, stroke and opacity.
- **Path**: Union, Intersect, Punch, Crop, Join, Split, Simplify, Expand Stroke, Inset and Outset.
- **Auto Shapes**: drag a tile onto the canvas, or click it to place the shape in the middle of the page, then adjust it in Auto Shape Properties.
- **Document Library**: New makes a symbol from the selection. Insert places it, Update redefines it from the selection, and Delete removes it.
- **Common Library**: drag an item onto the canvas, or click it, to add it to the document as a symbol. Symbols of the Document Library can be dragged onto the canvas too.
- **Find**: replace text or colors on this page or on all pages.
- **Color Palette**: click a color to fill with it, or Shift+click for the stroke. It also suggests complementary, analogous and triad colors for the current fill.
- **Symbol Properties**: rename the selected symbol, see how many copies exist, and break it apart.

The panel menu at the right of each tab row collapses or closes that group. Expanded Mode in the top bar switches between open panels and tabs only.

## Comparing exports

Preview shows the page as it will export. 2-Up puts the original next to it, and 4-Up compares the original with three export settings and their file sizes. Drag to pan in these views.

## Preferences and Keyboard Shortcuts

**Edit > Preferences** (Ctrl+U) sets the number of undo steps, how near the pointer picks an object, how near things snap, the size and color of the grid and the color of the guides. With text selected, Ctrl+U underlines instead. **View > Grid > Snap to Grid** (Ctrl+Alt+Shift+G) makes moved objects snap to the grid lines while the grid is shown.

**Edit > Keyboard Shortcuts** lists every menu command with its keys. Pick a command, press **Change**, then the new keys. Keys that belonged to another command are taken from it. **Remove** leaves the command without keys and **Reset** gives back the ones it came with. Changes are kept in the settings file.

## Symbols, buttons and animation

**Double-clicking** a copy of a symbol opens the symbol where it lies: its parts lie loose on the page, the rest of the page is locked, and whatever is drawn meanwhile joins the symbol. A double-click on an empty spot, or the **◀ Page** button above the document, ends the edit, and every copy of the symbol changes. **Modify > Symbol > Edit Symbol** does the same.

**Edit > Insert > New Symbol** (Ctrl+F8) asks for a name and a type (Graphic, Animation, Button) and opens the empty symbol for drawing. **New Button** (Ctrl+Shift+F8) opens a button symbol straight away. A button has four states: Up, Over, Down and Over While Down. While editing, pick the state from the list above the document; in an empty state **Copy … Graphic** brings the state before it. With a button selected on the page, **Link** in Properties takes the address a click leads to.

In **Preview** the button shows its Over state under the mouse and its Down state when pressed. On export each button gets a picture for every state and an HTML page (`name.html`) that swaps them.

**Modify > Animation > Animate Selection** (Alt+Shift+F8) turns the selection into an animation symbol: it asks for the number of states, the distance and direction of the move (0° right, 90° up), scaling, opacity and rotation, and adds states if needed. While selected its path shows (green start, red end); dragging the red end changes distance and direction, dragging the green one moves where it starts, and moving one of the copies moves the whole animation. **Settings** opens the same dialog again and **Remove Animation** leaves only the copy in the first state.

## Templates, batch processing and commands

**File > New from Template** lists ready page sizes, grid systems marked with guides, phone and tablet screens, and your own templates; **Browse** opens any document as a template. A document started from a template has no file, so saving asks for a name. **File > Save as Template** adds the document to your templates.

**File > Batch Process** scales, converts and renames many files at once: add the files, pick the scaling (to a size, to fit an area, to a percentage), optionally use this document's Optimize settings, give a prefix or suffix and pick the output folder. If a result would replace an existing file, the old one is first copied to an `Original Files` folder.

The **Commands** menu has ready-made commands: Creative (Add Arrowheads, Add Picture Frame, Add Shadow, Auto Vector Mask, Convert to Grayscale, Convert to Sepia Tone, Twist and Fade) and Document (Distribute to Layers, Hide Other Layers, Lock Other Layers, Reverse All States, Reverse State Range). **Save Steps as Command** in the **History** panel keeps the menu commands that changed the document under a name; saved commands show at the foot of the menu and are renamed or deleted with **Manage Saved Commands**. Steps made with the mouse (drawing, dragging) are not recorded. **Run Script** plays such a file (one `Menu > Command` per line); it does not run Fireworks' JSF files.

## Text orientation

With text selected, the orientation list in Properties offers four choices: **Horizontal left to right**, **Horizontal right to left** (the letters of a line are set in the opposite order), **Vertical left to right** and **Vertical right to left**. In vertical text each line is a column and the letters stand upright one under another. The alignment buttons work down the column for vertical text; Top, Centered Vertically, Bottom, Justified Vertically and Stretched Vertically in **Text > Align** do the same.

## Fireworks PNG files

Fireworks' own PNG files open with their layers through **File > Open**: layer names and visibility, states, bitmap objects and the Drop Shadow effect are read. Paths, text and groups cannot be read; a file that has any opens as its flat picture, and the status bar says so. What opens is a new document: saving never writes over the Fireworks file.

## Examples

The **Examples** menu at the right end of the menu bar is for what PisiWorks has of its own and Fireworks does not; the menus on the left stay as a Fireworks user knows them. Picking an example opens it as a new, unnamed document that you can change, save or export. The texts in the examples come in the language of the interface.

| Example | What it shows |
|---|---|
| Web banner | Gradient fill, text, a live filter and an Auto Shape |
| Navigation buttons | Button symbols with three states and links; try them with the mouse in Preview, and exporting writes an HTML page as well |
| Flying star | An animation symbol across eight states; play it from the States panel, export it as an animated GIF |
| School poster | Text on a path, a mask and live filters |
| Fills and gradients | The twelve kinds of gradient side by side |
| Live filters | One shape under eight filters |
| Web page mockup | A 12-column grid of guides, layers, slices and hotspots |

## Animated GIF

- **Opening.** An animated GIF opened with **File > Open** becomes a state for each of its pictures; the time each stays on show is in the States panel. What opens is a new document, and the GIF file is never written over.
- **Export and looping.** Pick **Animated GIF** in the Optimize panel. The **Looping** list on the same row sets how many times the animation plays: Forever, No Looping (once through) or a number of times.
- **Frames in between.** Select two or more copies of one symbol and choose **Modify > Symbol > Tween Instances** (Ctrl+Alt+Shift+T). As many copies as there are steps are made in between, going from one to the other in place, size, turn and opacity. With **Distribute to states** each copy goes to a state of its own, and the animation is ready.
- **Make a GIF.** **GIF > Make a GIF** at the right end of the menu bar makes an animation from picture files: add the files (they are put in order of name), pick the frame delay and the looping. **Save GIF** writes the file straight away; **Open as Document** lays the pictures out as states to go on working with. Pictures of different sizes are put in the middle of a canvas the size of the largest.

## Animating with PisiAnimasyon

PisiAnimasyon uses all of PisiWorks' drawing tools; what it adds is the timeline over the stage.

**The timeline.** Each layer has a row of frames. A click on a frame shows it; dragging along the numbers scrubs through the frames. The eye beside a layer's name hides it, the lock locks it.

| Key | Does |
|---|---|
| F5 | Insert frame (the keyframe lasts longer) |
| Shift+F5 | Remove frames |
| F6 | Insert keyframe (starting with what the frame shows) |
| F7 | Insert blank keyframe |
| Shift+F6 | Clear keyframe |
| Enter | Play once to the end, or stop |
| . and , | One frame forward, one back |
| Ctrl+Enter | Test Movie |

**Playing.** Enter plays the film once from the frame on show and stops on the last frame, as Flash does; pressed on the last frame it starts from the first. To have it go round and round, tick **Loop** under the timeline, or choose **Control > Loop Playback**. This is only about playing inside the program: the published film always starts over after its last scene, unless the last keyframe has a **Stop** action.

**Long films.** The blue marks on the ruler are whole seconds (at 24 frames a second, 20 seconds is frame 480). Under the timeline, the **Frame** box shows the frame on show and the box after it its time in seconds; type a number in either and press Enter to go straight there. The timeline scrolls sideways with the wheel, and follows the playhead while the film plays.

**Each layer has its own time.** A layer's keyframes are its own, so shapes can move at different times:

- *Moving, then standing still.* Make the motion tween between frame 1 and, say, frame 480. Then click a later frame of the same layer (frame 960) and press F5: the last keyframe lasts until there, and the shape stays where it arrived.
- *Starting late.* Add a new layer, click frame 480 in it and press F7 (blank keyframe), then draw the shape there. The layer shows nothing until that frame. Give it its own later keyframe (F6) and tween.
- *Leaving early.* On the frame where the shape is to go, press F7: from that blank keyframe on the layer is empty.

The **One after another** example shows all of this in eight seconds: a car drives for four seconds and stops, then the sun rises, then the words fade in.

**Behaviors: motion with no keyframes.** This is PisiAnimasyon's own; Flash has nothing like it. Select an object and pick a motion from the **Behaviors** menu at the right end of the menu bar (or from the right-click menu): the object has it from then on, on every frame, with no keyframes and no tween. Picking it again takes it away. An object can have several at once.

| Behavior | What the object does | Its numbers |
|---|---|---|
| Spin | Turns round and round its transformation point | Seconds for one turn (a small number spins fast: 0.5 is two turns a second); counterclockwise (1: yes, 0: no) |
| Bounce | Jumps up and comes down, again and again | Height, seconds for one bounce |
| Go Back and Forth | Goes right and comes back (a negative distance goes left) | Distance, seconds there and back |
| Go Up and Down | Rises and sinks | Distance, seconds there and back |
| Keep Going | Travels on without stopping; leaving one edge of the stage it comes in from the other | Speed, direction (0 right, 90 up) |
| Shake | Trembles | Strength |
| Grow and Shrink | Swells and goes back | Growth in percent, seconds for one beat |
| Blink | Shows and hides | Seconds for one blink |
| Fade In and Out | Fades away and comes back | Seconds there and back |
| Disappear After a While | Is gone when its time is up: for sparks and bursts | Seconds |
| Follow the Mouse | Goes after the mouse | Speed (0: stays on the mouse) |
| Walk with the Arrow Keys | The viewer moves it with the arrow keys; it stays on the stage | Speed |
| Fall and Jump | Falls to the bottom edge of the stage; Up or Space makes it jump | Pull, jump speed (0: no jumping) |
| Can Be Dragged | The viewer drags it with the mouse | |
| Bounce off the Edges | Flies in a straight line and comes back from every edge of the stage, and from whatever is solid, as a ball does | Speed, direction (degrees; 0 right, 90 up) |
| Roll Like a Wheel | Turns by as far as it goes: fast when it goes fast, slowly when it slows, not at all when it stands, backwards when it goes back | |
| Camera Follows Me | The view goes along with it: the object stays where it was drawn on the screen and everything else slides past | |
| Slide Like Scenery | Slides by only a part of how far the view has gone, as what is far away does; may come back in at the other side when it leaves the screen | Percent (100 as the road, 0 stays on the screen), comes back in at the other side (1: yes, 0: no) |
| Is Solid | Nothing that walks or falls passes through it: it stops at its sides, stands on its top and bumps its head on its underside | |

The first ten show when you play inside the program (Enter): the film then plays on, holding its last frame, until you stop it. The last nine have to do with the viewer, with the edges of the stage, with the way the object goes or with the camera, so they work in the published film: see them with **Test Movie** (Ctrl+Enter). **Behaviors > Behavior Settings** lists the selected object's behaviors with their numbers and a Remove button for each. Behaviors and tweens can be used together: a tweened object does both. They work on objects on the stage, not on those inside a symbol or a group; give the behavior to the copy of the symbol or to the group itself.

*Smoothness.* In the published film everything that behaviors and rules move is drawn as often as the screen is (sixty times a second on most screens), whatever the film's frame rate; so what walks, falls and drifts moves smoothly. Drawings made with keyframes and tweens change at the film's own rate: to smooth those, raise the frame rate (the **fps** box under the timeline) to 30. Played inside the program with Enter, everything goes at the film's rate.

*A first game in two minutes:* draw a box, give it **Walk with the Arrow Keys** and **Fall and Jump**; draw a star and give it **Spin**; press Ctrl+Enter. The **Behaviors** example has every kind on a single frame.

**Rules: from a film to a game.** Also PisiAnimasyon's own. A rule says *when this happens, do that*, and both halves are picked from lists: nothing is typed but numbers. **Rules > Rules and Numbers** shows every rule as a sentence; click one to take it apart.

- *Names first.* A rule knows an object by its name. The name is typed into the **Name** box at the left of the **Properties** panel while the object is selected (Enter takes it); it is the same for anything drawn: an Auto Shape, a shape drawn by hand, a group. With a copy of a symbol selected, the **Symbol Properties** panel that opens has the same **Name** box; the name there is the copy's, while the one in the **Symbol** box is that of the symbol in the library, which rules do not go by. A shape just drawn has the name of its kind ("Ellipse", "Gear"); give it a name of its own before writing a rule. Name the apple "Apple" and the basket "Basket". Objects with the same name share their rules: one rule about "Apple" holds for every apple.
- *New Rule* lists the events:

| Event | Happens |
|---|---|
| When the film starts | Once, at the beginning and after "start again" |
| When a scene starts | At the beginning, and whenever the film goes to another scene |
| When a key is pressed | Space, the arrows, Enter, a letter or a digit goes down |
| When an object is clicked | The viewer clicks an object of that name |
| When an object touches another | The boxes of the two come to overlap |
| When an object touches the edge of the stage | Any edge, or the bottom, top, left or right one |
| Every so many seconds | Again and again |
| After so many seconds | Once |
| When a number reaches a value | It becomes, goes above or goes below the value |

- *Add a Step* lists what to do: change or set a number, or set it at random; hide, show or move an object, put it at a place, somewhere at random, or at the top at random; make a copy of an object where another is; remove an object; make an object blink for a moment; make an object faster or slower; make it jump; turn it by degrees or keep it spinning, resize it, make it see-through; give it a behavior or take one from it; send an object after another; set an object going, let it slow down and stop, or stop it at once; attach an object to another or let it go; make the camera follow an object or stand still; play a sound; go to the next scene, to a scene or to a frame; stop or play the film; stop everything; start again. A rule can have several steps; they are done in order.
- *Only if.* Under the event, **Only if** can hold a number against a value: "When Hero touches Door, if Score is more than 2". The rule is then followed only when that is so as well.
- *Copies.* **Make a copy of an object** makes another of the same name, with the same behaviors, where a second object is: a bullet where the ship is. Draw the one bullet, and hide it when the film starts; every shot is a copy. **Remove an object** takes a copy away for good, and hides anything else until the film starts again.
- *A copy for each.* When the second name is that of several objects, each gets a copy: "Every 2 seconds: make a copy of Shot at Rat" makes every rat shoot. A gate is an object drawn just off the stage for copies to appear at: "Every 3 seconds: make a copy of Rat at Gate".
- *Blinking.* **Make an object blink for a moment** shows that it was hit. While it blinks nothing can touch it, which gives the player a moment to get away.
- *Faster.* **Make an object faster** quickens everything the object does by itself: falling, going, turning. This is how a game gets harder as it goes.
- *Touching.* Two objects touch when their boxes meet; an ellipse that has not been turned is taken by its outline. What stands on a solid object touches it.
- *Which object?* When a rule says "When Apple touches Basket: hide Apple", it is the apple that touched that is hidden, not every apple.
- *Numbers.* **New Number** makes a number with a name and a starting value: a score, lives, a clock. A text with the name in braces shows it in the film: type `Score: {Score}` with the Text tool.
- *A thing happens once.* A rule is followed when something comes to be, not for as long as it lasts: an apple lying on the ground has touched it once.
- Rules work in the published film: **Test Movie** (Ctrl+Enter). A scene that is to wait for the player needs the frame action **Stop**, or the film goes on to the next scene.

**Lesson: catch the apples** (the example **Game: catch the apples**).

1. Draw a basket at the bottom of the stage and name it "Basket". Give it **Walk with the Arrow Keys** and **Fall and Jump** with a jump speed of 0: falling keeps it on the ground.
2. Draw an apple at the top and name it "Apple". Give it **Fall and Jump** with a small pull (70) and a jump speed of 0. Copy it for a second apple, higher up.
3. In **Rules and Numbers** make two numbers: Score, starting as 0, and Lives, starting as 5. Type `Score: {Score}` and `Lives: {Lives}` on the stage.
4. Rule: *When an object touches another*, Apple, Basket. Steps: *Change a number*, Score, by 1; *Put an object at the top, at random*, Apple.
5. Rule: *When an object touches the edge of the stage*, Apple, the bottom edge. Steps: *Change a number*, Lives, by −1 (the sentence reads "decrease Lives by 1"); *Put an object at the top, at random*, Apple.
6. Add a second scene saying "Game over", with a box named "Again". Give both scenes the frame action **Stop**.
7. Rule: *When a number reaches a value*, Lives, goes below, 1. Step: *Go to a scene*, 2.
8. Rule: *When an object is clicked*, Again. Step: *Start again*.
9. Ctrl+Enter.

**Lesson: hit the targets.** Three circles named "Target", each with a behavior of its own (Go Back and Forth, Go Up and Down, Keep Going). Numbers: Score 0 and Time 20. Rules: *When Target is clicked: increase Score by 1; put Target somewhere at random*. *Every 1 seconds: decrease Time by 1*. *When Time goes below 1: go to scene 2*.

**Lesson: quiz.** One scene for each question, each with the frame action Stop. The answers are boxes with words on them; name the right one "Right answer" and the others "Wrong answer". Three rules do for any number of questions: *When Right answer is clicked: increase Score by 1; go to the next scene*. *When Wrong answer is clicked: go to the next scene*. *When Again is clicked: start again*. The last scene shows `Score: {Score}`.

**Lesson: a platform game** (the example **Game: platform**).

1. Draw the ground and three platforms, each a rectangle, and give each **Is Solid**.
2. Draw the hero, name it "Hero", and give it **Walk with the Arrow Keys** and **Fall and Jump**. Test it: it walks, jumps, lands on the platforms and bumps its head under them.
3. Put a coin over each platform; name each "Coin". Rule: *When Hero touches Coin: increase Score by 1; remove Coin*.
4. Draw an enemy on the ground, name it "Enemy" and give it **Go Back and Forth**. Rule: *When Hero touches Enemy: decrease Lives by 1; put Hero at 30, 322* (where the hero started; its X and Y are in Properties).
5. Draw a door and name it "Door". Rule: *When Hero touches Door*, **Only if** Score is more than 2: *go to scene 2*.
6. Rule: *When Lives goes below 1: go to scene 3*. Scene 2 says "Well done", scene 3 "Game over"; each has a box named "Again" and the frame action Stop.

**Lesson: a space shooter** (the example **Game: space shooter**).

1. Draw the ship at the bottom, name it "Ship", and give it **Walk with the Arrow Keys** and **Fall and Jump** with a jump speed of 0.
2. Draw one bullet, name it "Bullet", and give it **Keep Going** with direction 90 (up). Rule: *When the film starts: hide Bullet*.
3. Rule: *When the Space key is pressed: make a copy of Bullet at Ship*. Rule: *When Bullet touches the top edge of the stage: remove Bullet*.
4. Draw four aliens, each named "Alien", each higher than the one before, and give each **Fall and Jump** with a small pull (22) and a jump speed of 0.
5. Rule: *When Bullet touches Alien: increase Score by 1; remove Bullet; put Alien at the top, at random; make Alien faster by 6 percent*.
6. Rule: *When Alien touches the bottom edge of the stage: decrease Lives by 1; put Alien at the top, at random*. Rule: *When Lives goes below 1: go to scene 2*.

**Making a game feel alive.** The example games use the same few rules as the lessons, and four things more. They are what makes a game worth playing, and each is a click or a sentence:

- *Characters.* Draw a character from several shapes (a body, eyes, a cap), select them all and group them (Ctrl+G), and name the group. The group is one object to rules and behaviors. Open an example and ungroup its hero to see how it is made.
- *A burst where something happens.* Draw a star, name it "Sparkle", and give it **Grow and Shrink**, **Spin** and **Disappear After a While** (0.35 seconds). Rule: *When the film starts: hide Sparkle*. Then add to the rule of the coin: *make a copy of Sparkle at Coin*. Every coin taken flashes.
- *A sound for each thing.* Import a short sound (**Import Sound**) and add *Play a sound* to the rule. The examples carry five sounds made for them: ding, zap, boom, oops and fanfare.
- *Scenery that moves.* Clouds with **Keep Going**, a sun with **Spin**, stars streaming past at three speeds, lamps with **Blink**. None of it is in a rule.
- *Harder as it goes.* Add *make … faster by 5 percent* to the rule that gives a point.

**Where the characters are, and how to change one.** To change a character of an example, **double-click** it with the Pointer tool (the black arrow). What happens depends on what the character is:

- *A group.* Most characters of the small games are groups lying on the stage. A double-click goes inside the group: its parts can be picked and changed one by one (the color of the hero's cap, the size of its eyes). A click beside the group closes it. **Modify > Ungroup** (Ctrl+Shift+G) takes it apart altogether.
- *A symbol.* Every character of Pisi: Iron Claw is a symbol: the rat, the drone, the shot, the bomb, the heart, the bullet, the explosion, the turret, the giant rat. A double-click opens the symbol where it lies, and the name of the symbol being edited shows above the stage. To finish, do one of three things: double-click an empty place, press Esc, or click the **◀** button above the stage. If a skew or a scale is still being made, the first Esc gives it up and the second leaves the symbol; the double-click and the button apply it and leave. A change made in the symbol shows **in every scene** and in every copy of the character: change the rat once, and every rat of every stage is changed. The symbols are listed in the **Document Library** panel.
- *A movie clip.* A character that moves its legs is a movie clip: Pisi is one. A double-click opens the clip; its timeline has the four steps of the run, each a drawing of its own. The **◀** button above the stage goes back.

*Around the stage.* The grey area around the stage is the pasteboard, and PisiAnimasyon shows what lies on it, as Flash does. What copies are made from is kept there: in Pisi: Iron Claw they stand in a column off the left edge, and the gates, small pink stars, off the right and the top. Zoom out, or scroll, if they are out of sight.

*To keep in mind.* A character reshaped with the Scale, Skew or Distort tool is changed when you press Enter or double-click, and only that copy in that scene is. To change the character itself everywhere, change the drawing in its symbol. Not every character comes into play in every scene: the drone first comes in the Factory, so a change to it does not show in the first stage. Do not change the names: the rules go by them.

**The showcase: Pisi: Iron Claw.** The first entry of the Examples menu is a game of some length: Pisi, a robot leopard with a blaster, goes through a city, a factory and a rooftop, and meets a giant rat at the end. Robot rats roll in and shoot, drones drop bombs, a turret guards the roof, hearts fall now and then. It is made of nothing the lessons do not teach, only more of it, and it is worth taking apart:

- *Pisi runs.* Pisi is a movie clip of four steps, three frames each. A copy of the clip on the stage has the behaviors **Walk with the Arrow Keys** and **Fall and Jump**: a clip can have behaviors and a name like anything else.
- *The world comes to Pisi.* PisiAnimasyon has no camera. Each layer of scenery is a strip as wide as the stage with **Keep Going** to the left, and a second strip after it; the far ones go slowly and the near ones fast, so the town seems to pass by.
- *Enemies come from a gate.* One rat is drawn off the left edge and hidden when the scene starts. *Every 2.6 seconds: make a copy of Rat at Gate*. *Every 2.3 seconds: make a copy of Shot at Rat*. *When Bullet touches Rat: increase Score by 1; make a copy of Explosion at Rat; remove Bullet; remove Rat*.
- *Stages.* *When Score becomes 10: go to scene 2*. Every scene has objects of the same names, so the same rules hold in all of them; a scene with no Sky gate has no drones.
- *The giant rat.* A number, Armor, starts at 30. *When Bullet touches Giant Rat: decrease Armor by 1; … make Giant Rat blink for a moment*. *When Armor goes below 1: go to scene 5*.

All twenty-eight rules can be read in **Rules > Rules and Numbers**.

What rules do not do yet: a moving platform does not carry what stands on it; touching goes by boxes, except for ellipses; a number is whole or decimal, there are no words to keep; in the lists a rule has one "only if"; several conditions and "otherwise" are in the written language. Levels larger than the stage, a camera and a character with several animations belong to a game studio, not to rules.

**Writing the rules: the language.** Every sentence of Rules and Numbers can be typed as well. **Rules > Rules as Text** opens all the rules and numbers as a text; a game made with the lists stands here written out, and what is written here shows in the lists. They are two faces of one thing.

```
Numbers:
    Score = 0
    Lives = 5

When Apple touches Basket:
    increase Score by 1
    put Apple at the top, at random

When Lives goes below 1:
    go to scene 2
```

- *The shape.* A rule is a line for its event, starting at the edge and ending with a colon, and its steps under it, written further in. Enter after an event line puts the caret further in by itself. Under the heading `Numbers:` each number is written `Name = value`. A line that starts with `#` is a remark and is not read.
- *Read as it is written.* The text is read again at every key. A line that cannot be read turns red, and the reason stands below with its line: "Line 7: this is not a step I know", "Line 9: there is no number called Score". A click on the reason takes the caret to the line. A name of an object or a sound that the document has not got is shown as a warning; the text is taken all the same.
- *Put in a sentence.* The sentences need not be known by heart: the lists **An event** and **A step** at the top write one, and you put a name or a number in place of each `?`.
- *Applying it.* **Apply** (Ctrl+S) takes what is written into the document; **Apply and Test** (Ctrl+Enter) takes it and opens the game in the browser. The text is not applied while a line cannot be read. Esc closes the window.
- *Pasting.* Ctrl+V pastes rules copied from elsewhere: the rules of a game can be sent to a friend as text.
- *Taking back.* Ctrl+Z takes the last change back, Ctrl+Y (or Ctrl+Shift+Z) puts it in again. Letters typed one after another are one change: a word comes back out whole.
- *What reads but will not work.* Where a text reads and still will not do what is hoped, the window says so on a yellow line, and where the thing can be put right it puts a button beside it that does. Beside "Top cannot jump yet: it needs the behavior Fall and Jump", **Give It Fall and Jump** gives that behavior to the objects called Top without leaving the window. Beside "Nothing in the document is called kTop. Did you mean Top?", **Write Top** puts the name right in the text. A jump too small to see is told of in the same way.

- *Offered as you write.* When a line is begun, offers appear under the caret: the sentences the line can be, and the names of the document. They go by what is typed: the name of an object brings the sentences about objects (`hide Apple`), in Turkish the name of a number brings those about numbers, a figure those that open with one; a name typed in small letters is put in as it is kept. The up and down arrows pick, **Tab** takes. When a sentence is taken the caret goes to its first gap; as the gaps are filled, Tab goes on to the next. Esc waves the offers away.

What the writing has over the lists:

- *Sums.* Wherever a number goes, a sum may be written: `set Score to Score * 2`, `increase Lives by Score / 10`. The four signs and brackets are used. The place of an object is read with `Name.x` and `Name.y`: `put Bullet at Ship.x + 20, Ship.y`. A whole number at random is `random(1, 6)`. An object with a space in its name is written between quotation marks: `"Giant Rat".x`. (The number boxes of the lists take sums too.)
- *Several conditions.* Conditions are joined by `and`: `When Hero touches Door, if Score is more than 2 and Lives is more than 0:`. The rule is followed when all of them hold.
- *Otherwise.* After the steps, `otherwise:` is written at the edge; what stands under it is done when the conditions do not hold.
- *Waiting.* `wait 1 seconds` puts off the steps after it: the sound first, the change of scene a second later.
- *Repeating.* `repeat 3 times:` does the steps written further in under it three times; the count may be a sum (`repeat Shots times:`). With a `wait` inside, it waits between rounds.
- *If.* Inside a rule, what is under `if Score is more than 10:` is done only when that holds; `otherwise:`, written as far in as the `if`, says what to do when it does not. Blocks can be written inside blocks.
- *Repeating while.* `repeat while Score is more than 0:` does what is under it for as long as that holds. A loop that would never end does not hold the game up: the program cuts it short.
- *Commands.* A thing done often is given a name: `The command Burst:` at the edge, its steps under it; then any rule says `do the command Burst`. A command may be used above where it is written, and may do other commands, itself among them.
- *Giving a command values.* A command can take values: `The command Award with Points, Who:` defines one that takes two; `do the command Award with 10, Coin` does it with those. Inside the command the name of a value is used as a number (`increase Score by Points * 2`) or stands for the object given (`hide Who`). A value may be a number, a sum, or the name of an object; an object with a space in its name is written between quotation marks. A command is given as many values as it takes; too few or too many, and the line turns red and says why. Changing a value inside the command (`decrease Points by 1`) holds for that run of it only.

```
When Hero touches Door, if Score is more than 2 and Lives is more than 0:
    play the sound fanfare
    wait 1 seconds
    go to scene 2
otherwise:
    play the sound oops
    put Hero at Hero.x - 40, 322
```

```
Numbers:
    Score = 0
    Shots = 3

The command Burst:
    make a copy of Explosion at Enemy
    play the sound boom

When the Space key is pressed:
    repeat Shots times:
        make a copy of Bullet at Ship
        wait 0.2 seconds
    if Score is more than 10:
        do the command Burst
    otherwise:
        play the sound zap
```

```
The command Award with Points, Who:
    increase Score by Points * 2
    make Who blink for a moment

When Coin is clicked:
    do the command Award with 10, Coin
```

The lines that open a block: `repeat … times:`, `repeat while …:`, `if …:`, `otherwise:`. Defining a command: `The command …:` or `The command … with …, …:`. Doing it: `do the command …` or `do the command … with …, …`.

The sentences for events: `When the film starts`, `When a scene starts`, `When the … key is pressed`, `When the … key is released`, `When … is clicked`, `When … touches … of the stage`, `When … touches …`, `Every … seconds`, `After … seconds`, `When … goes above …`, `When … goes below …`, `When … becomes …`.

*Turning, resizing, giving behaviors.* `turn` turns the object by so many degrees, clockwise, around its transformation point, adding to how far it is turned already (a number below zero turns it the other way); it happens at once, so turning by 360 degrees shows nothing. To keep it turning write `make … spin … turns a second`: "When Button is clicked: make Fan spin 3 turns a second" starts the fan; a number below zero spins it counterclockwise, 0 stops it. The speed may be a number of the film: `make Fan spin Speed turns a second`. `percent of its size` and `percent opaque` count from the object as it was drawn: 100 percent is as drawn, 200 twice the size, 0 percent opaque cannot be seen. `give … the behavior` gives one of the behaviors of the Behaviors menu, by its name and with the numbers it starts with, at that moment: "When Button is clicked: give Wheel the behavior Spin" starts the wheel spinning, and `take from` stops it (what was spinning stays turned as it was). `send … after` sends one object towards another at so many pixels a second; a speed of 0 calls it off.

*Motion that gathers speed and loses it.* `make Car go to the right at 300` sets the car going: it reaches that speed in under half a second and keeps going. The way is `to the right`, `to the left`, `upwards` or `downwards`; the speed is in pixels a second. `make Car slow down and stop` takes its speed away little by little, so that it stands still in a second and a half; `make Car stop` stops it at once. Two rules make it go while a key is held and slow down when it is let go: one for `When the Right Arrow key is pressed`, one for `When the Right Arrow key is released`.

*Attaching one object to another.* The menu is the easy way: select the object to attach, choose **Behaviors > Attach to Another Object…** and click, in the list that opens, the name of the object it is to go along with (that object must have a name, given in Properties). **Attached to nothing**, in the same list, unties it. A rule can attach during the game as well: after `attach Wheel to Car` the wheel goes wherever the car goes, so only the car has to be moved. What is attached keeps its own behaviors: a wheel with **Roll Like a Wheel** turns by as far as the car goes. `let … go` unties it, and it stays where it is. What is attached follows the other's place only, not its turning or its size.

*The camera and scenery that slides.* Give an object **Behaviors > Camera Follows Me**: in the published film it stays where it was drawn on the screen, and as it goes right the world slides left. The camera goes sideways only. Three rules say what slides and how:

- Drawings with no behavior that no rule names (the sky, the ground) become one picture, and it **stays on the screen**.
- Every object with a behavior, or named in a rule, is **part of the world**: it slides by as far as the camera goes and leaves the screen (an enemy, an obstacle, a door).
- An object with **Slide Like Scenery** slides by its percent: 100 for road marks and bushes by the road, 20 to 40 for hills, 5 to 10 for clouds. What is far slides little and what is near slides much, which is parallax. With "comes back in at the other side" on, an object that leaves one end of the screen comes in at the other, so the road never ends. Zero percent keeps an object on the screen: a turning sun, or a life gauge.

Texts that show a number (`Score: {Score}`) always stay on the screen. With a camera, "the edge of the stage" is the edge of the screen: a shot that leaves the screen can be removed by a rule about the right edge. Draw what repeats inside the stage, and make such objects the same width so that they stay evenly spaced: each comes round once in a stage's width plus its own. Rules can do it too: `make the camera follow Car` hands the view to that object there and then, without a leap, and `let the camera stand still` lets it go. Playing inside the program with Enter does not move the camera; see it with **Test Movie**.

**Lesson: a car.** (The example **Driving a car**.) Draw the body, group it and name it "Car". Keep the wheels out of the body's group: draw two wheels of their own, name both "Wheel" and give both **Roll Like a Wheel** (draw spokes on the wheel, so that its turning shows). Select the two wheels and attach them to "Car" with **Behaviors > Attach to Another Object…**. The rules:

```
When the Right Arrow key is pressed:
    make Car go to the right at 320

When the Right Arrow key is released:
    make Car slow down and stop
```

Write the same two rules with `to the left` for the left arrow. The car gathers speed, and when the key is let go it slows down and stops; its wheels slow down with it.

To have the road slide, give the car **Camera Follows Me**. Give the road marks and the bushes **Slide Like Scenery** at 100 percent, the hills at 20 to 45, the clouds at 5 to 10. Give the sky and the tarmac nothing: they stay where they are. Now the car stands on the screen and the road runs under it, slowing when the car slows.

The sentences for steps: `move … by … right and … down`, `put … somewhere at random`, `put … at the top, at random`, `put … at …, …`, `show …`, `hide …`, `make … jump … high`, `turn … by … degrees`, `make … spin … turns a second`, `make … … percent of its size`, `make … … percent opaque`, `give … the behavior …`, `take from … the behavior …`, `send … after … at …`, `make … go … at …`, `make … slow down and stop`, `make … stop`, `attach … to …`, `let … go`, `make the camera follow …`, `let the camera stand still`, `decrease … by …`, `increase … by …`, `make a copy of … at …`, `remove …`, `make … blink for a moment`, `make … slower by … percent`, `make … faster by … percent`, `set … at random from … to …`, `set … to …`, `wait … seconds`, `stop the film`, `play the film`, `go to frame … and play`, `go to frame … and stop`, `go to the next scene`, `go to scene …`, `play the sound …`, `stop everything`, `start again`.

Conditions: `… is …`, `… is more than …`, `… is less than …`. The names of keys: Space, Up Arrow, Down Arrow, Left Arrow, Right Arrow, Enter, the letters and the digits. Edges: any edge, the bottom edge, the top edge, the left edge, the right edge.

With the program in Turkish the same rules are written and read in Turkish sentences (`Elma nesnesi Sepet nesnesine değince:`); the rules are kept in the document apart from any language, so a game written in one shows in the other in that language's sentences. The names of objects and numbers are not translated.

In the lists (Rules and Numbers) blocks and commands are shown but not made; they are written. What the language does not do yet: a command gives no value back (it cannot be used inside a sum); nothing but numbers is kept (no words, no lists); only the place of an object is read, not its size or whether it shows. The writing area is a plain editor: there is no selecting and no find and replace; a long text can be written in an editor of your own and pasted with Ctrl+V.

**Turning at a joint.** A motion tween turns an object around its *transformation point*, as in Flash. It starts in the middle of the object. To move it, select the object, take the **Scale** tool (Q) and drag the small square in the middle of the box to where the object is to turn: the shoulder end of an arm, the hub of a wheel, the hinge of a door. Press Enter. The point stays with the object from then on, through moves, turns and new keyframes, and the tween keeps it going straight from keyframe to keyframe while the object turns around it. Put the point in place on the first keyframe, before inserting the others. A group or a picture should be made a symbol first (F8), as Flash asks for any motion tween.

**Lesson: a stick-figure fight.** Open **Examples > Stick fight** and play it (Enter), then take it apart. This is how such a film is made:

1. *The figure.* On a layer of its own draw the figure with the Line tool, thick strokes: one line for the body, two for each arm (shoulder to elbow, elbow to hand), two for each leg, and an ellipse for the head. Draw each line starting from the joint it hangs from.
2. *The joints.* With the Scale tool move each line's transformation point onto that joint: the upper arm's onto the shoulder, the forearm's onto the elbow, the thigh's onto the hip, the shin's onto the knee.
3. *The poses.* Think of the film as a few strong poses: on guard, arm drawn back, punch, back on guard. Put a keyframe (F6) where each pose is to be, and on it turn the lines into the pose with the Scale tool (drag outside the box to turn). Turn the upper arm first, then move the forearm back onto the elbow and turn it.
4. *The timing.* A blow is fast: two frames from drawn back to punch, then hold the punch for four frames. Waiting is slow: ten frames from one guard pose to a slightly lower one and back. Uneven timing is what makes it look alive.
5. *The frames between.* Select the keyframes' frames and choose **Create Motion Tween**. Between poses that are far apart, add a keyframe halfway and correct it: the forearm goes straight from keyframe to keyframe while the elbow goes round, so over a wide swing they drift apart. The example has a keyframe on every second frame, and on every frame during a blow.
6. *The enemies.* Each on its own layer, and each with its own time: a blank keyframe (F7) where it has not come yet, and another where it has gone.
7. *The effects.* A star on its own layer that grows and fades over four frames where the blow lands, and a word beside it.
8. *Test Movie* (Ctrl+Enter) to see it in the browser.

**Right-click.** A right-click on a frame goes to it and opens a menu of the tween, frame and keyframe commands; those that do not fit the frame are greyed. A right-click on a layer's name opens the layer commands (new layer, delete, guide, mask, tie). A right-click on an object on the stage selects it and opens Cut, Copy, Arrange, Transform, Group, Convert to Symbol and the like; PisiWorks' canvas has the same menu.

**Motion tween.** Draw the object on the first frame, insert a keyframe further on with F6 and move, resize or turn the object there; go back to the first keyframe and choose **Timeline > Create Motion Tween**. The frames between turn blue. **Ease** (−100 starts slowly, 100 ends slowly) and **Turns** (whole turns on the way) appear below. A turning wheel can be made two ways: turn the shape on the second keyframe (with the Scale tool, or Transform > Rotate 90), or leave the two keyframes alike and put 1 in **Turns**, which takes it once round.

**Shape tween.** Draw a shape on the first frame and another on a blank keyframe (F7) further on; on the first frame choose **Create Shape Tween**. The frames between turn green. If the change does not go as you want, add marks with **Add Shape Hint** (Ctrl+Shift+H): drag the yellow mark on the first keyframe and the green one on the last to the points that are to become one another.

**Guide layer.** Add a layer above the moving object's layer, draw a path in it, and choose **Guide Layer**. The object follows the path, and with **Orient to path** it turns to face the way the path runs. The guide does not show in the published film.

**Mask layer.** Add a layer above the one to be masked, draw a shape in it and choose **Mask Layer**: the layer beneath shows only where that shape is. The mask is applied in Preview, when publishing, and on the stage once both layers are locked. **Tie to Guide or Mask** ties further layers to the same guide or mask.

**Symbols.** **Edit > Insert > New Symbol** offers three kinds. A *Graphic* is edited in place. A *Movie Clip* has a timeline of its own: a double-click on a copy opens the clip's page, and the **◀** button above the stage goes back. A *Button* has Up, Over and Down states.

**Actions.** The **Frame action** list under the timeline says what happens when the film reaches that keyframe; a keyframe with an action shows red. With a button selected, the **Action** list in Properties says what a click does. The actions are Stop, Play, Go to Frame and Play, Go to Frame and Stop, Go to Scene and Go to Address. Frames and scenes are counted from 1.

**Scenes.** Every page in the Scenes panel is a scene. The film plays them one after another and starts over after the last.

**Sound.** **Import Sound** adds an MP3, OGG or WAV file to the document and starts it on the keyframe you are on; a keyframe with a blue rim has a sound. While playing inside the program the sound is heard if `ffplay`, `mpv`, `paplay` or `aplay` is installed.

**Publishing.** **Control > Publish as HTML5** writes the film as a single HTML file; **Test Movie** makes the same file in a temporary place and opens it in the browser. Movie clips, buttons, actions and sound work only in the published film; while playing inside the program clips show their first frame.

**Examples and exchange with PisiWorks.** The **Examples** menu at the right end of the menu bar has sixteen ready films (first the showcase, Pisi: Iron Claw; the five small games drawn with characters, scenery, bursts and sounds), each showing one thing (eased motion, shape change, a guide, a mask, a movie clip, layers with times of their own, a stick-figure fight made pose to pose, behaviors, five games made with rules, scenes and buttons). **File > Open Frame in PisiWorks** opens the frame on show in PisiWorks as a copy; **File > Open in PisiAnimasyon** in PisiWorks does the reverse. The copies are written to the cache folder, and your own document is left as it is. To bring artwork from one program into the other, choose a `.pisi` or `.sahne` file with **File > Import**: its objects come onto the active layer and its symbols into the library.
