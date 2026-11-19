<h1 align="center">PisiWorks</h1>

<p align="center">
  <strong>An open-source Adobe Fireworks look-alike for Pardus, with a Turkish interface.</strong><br>
  For the teachers who still teach Fireworks, and their students.
</p>

<p align="center">
  <a href="COPYING"><img src="https://img.shields.io/badge/license-GPL--3.0--or--later-ff6a2b" alt="GPL-3.0-or-later license"></a>
  <img src="https://img.shields.io/badge/Rust-1.91%2B-1b2042?logo=rust" alt="Rust 1.91 or later">
  <img src="https://img.shields.io/badge/UI-GPUI-7a5cff" alt="Built with GPUI">
  <img src="https://img.shields.io/badge/platform-Linux-2ed3b7" alt="Linux">
</p>

<p align="center">
  <a href="#get-started">Get started</a> ·
  <a href="#features">Features</a> ·
  <a href="docs/guide.md">User guide</a> ·
  <a href="README.md">Türkçe</a>
</p>

<p align="center">
  <img src="docs/screenshots/pisiworks-calisma-alani.png" alt="The PisiWorks workspace in Turkish, with the tools on the left, Properties below, the panel dock on the right and a landing page design in the middle">
</p>

> **PisiWorks is developed by Murat SAPMAZ** as an independent free-software project for schools in Turkey and for Pardus users. Its foundation is [Emberframe](https://github.com/gregstein/emberframe) by [Med Amine Belbachir](https://medamineweb.com/): without it this project could not have existed so soon, and we thank him. About a third of today's code comes from Emberframe and two thirds were written for PisiWorks; [what was done is listed below](#what-was-done-in-pisiworks).

## Purpose

**This project has no commercial purpose.** PisiWorks and PisiAnimasyon are free of charge and will stay so: they are not sold, show no advertising, ask for no donations and collect no data. Their one purpose is that teachers and students on Pardus, in schools whose curriculum still teaches Fireworks and Flash, can do the lesson on their own computers. They do not aim to take the place of a product of Adobe's, but to let the lesson go on where those tools do not run.

## Why PisiWorks

Adobe stopped developing Fireworks in 2013, but the program that mixed vector shapes, bitmap editing, states and web export in one window is still taught in schools. PisiWorks aims to offer the same way of working on the [Pardus](https://www.pardus.org.tr/) operating system, in Turkish, as free software.

It is deliberately a look-alike: the same docked layout, tool groups, panels and light gray interface, and in Turkish the short menu and command names are kept the same as the terms of the Turkish edition of Fireworks, so that a step in a teacher's notes is the step on the student's screen. Messages, descriptions and documents were written for this project. It is published under its own name, every icon is newly drawn, and it contains no Adobe code or artwork.

## Features

| Area | What you get |
|---|---|
| **Tools** | Pointer, Select Behind, Subselection, Scale, Skew, Distort, Crop, Marquee, Oval Marquee, Lasso, Polygon Lasso, Magic Wand, Brush, Pencil, Eraser, Blur, Sharpen, Dodge, Burn, Smudge, Rubber Stamp, Replace Color, Red Eye Removal, Eyedropper, Paint Bucket, Gradient, Line, Pen, Vector Path, Rectangle, Ellipse, Polygon, Text, Freeform, Knife, Hotspot, Slice, Hand, Zoom |
| **Panels** | Optimize, History, Align, Pages, States, Layers, Styles, Color Palette, Swatches, Path, Auto Shapes, Document Library, Common Library, Color Mixer, Info, URL, Find, Special Characters, Image Editing, Properties, Symbol Properties |
| **Vector** | Bezier curves and point editing, Auto Shapes, Union, Intersect, Punch, Crop, Join, Split, Simplify, Expand Stroke, Inset and Outset |
| **Transforming** | Scale, Skew, Distort, a movable center point, Numeric Transform, rotating and flipping |
| **Appearance** | Gradient fills of 12 kinds and pattern fills, 45 blend modes, masks, 29 live filters including Levels and Curves, Photoshop Live Effects with ten effects, styles |
| **Bitmap** | Paint, erase, clone, flood fill, pixel selections, Feather, Select Similar and Inverse, the marquee commands, Flatten Selection and Flatten Layers |
| **Text** | Installed fonts, bold, italic, underline, alignment, Convert to Paths |
| **Layout** | Rulers, grid, guides, smart guides, slice guides, groups with Superselect and Subselect |
| **Screens and animation** | Pages, layers, shared layers, states, onion skinning, symbols and a library |
| **Export** | PNG 8, PNG 24, PNG 32, JPEG with selective quality and smoothing, GIF and animated GIF, with Preview, 2-Up and 4-Up comparison, slices and HTML image maps |
| **Documents** | Native `.pisi` files (`.ember` files from Emberframe open too), opening and importing common image formats, Canvas Size, Image Size, Fit Canvas and Trim Canvas |
| **Printing** | Page Setup, printing to an installed printer or saving as PDF |
| **Language** | Turkish and English, changed from Edit > Language |

<p align="center">
  <img src="docs/screenshots/pisiworks-menuler.png" alt="The Modify menu and its Arrange submenu, with Turkish command names">
</p>

## Get started

Building and packaging are done on Linux Mint 22.3. The `.deb` package **has been installed and run on Pardus 25**, the system it is meant for (November 2026).

### Installing a package

A ready-made package is on the [releases page](https://github.com/MuratSAPMAZ/PisiWorks/releases) and is installed with `sudo apt install ./pisiworks_0.2.0_amd64.deb`; the release notes (in Turkish) are in [`docs/surum-notlari/`](docs/surum-notlari/).

One command also builds the package from source (the build requirements below must be installed):

```bash
packaging/build-deb.sh
```

The package is written to `target/deb/` and installed with:

```bash
sudo apt install ./target/deb/pisiworks_0.2.0_amd64.deb
```

After that PisiWorks is in the Graphics section of the application menu, and `.pisi` files open in it on a double-click. The maintainer written into the package is taken from the `DEB_MAINTAINER` environment variable, or failing that from git's `user.name` and `user.email`.

### Building from source

You need Rust 1.91 or later (from [rustup](https://rustup.rs/)) and these packages:

```bash
sudo apt install build-essential pkg-config libxcb1-dev libxkbcommon-dev libxkbcommon-x11-dev
```

The program draws with Vulkan. If the graphics driver has no Vulkan support, the software driver will do:

```bash
sudo apt install mesa-vulkan-drivers
```

To build and run:

```bash
git clone https://github.com/MuratSAPMAZ/PisiWorks.git
```

```bash
cd PisiWorks
```

```bash
git checkout pisiworks
```

```bash
cargo run --release
```

The first build takes a while. Then open [`examples/solstice-landing.pisi`](examples/solstice-landing.pisi) with Ctrl+O to explore the design in the screenshots. The [user guide](docs/guide.md) explains the tools and panels.

To run the tests:

```bash
cargo test
```

Printing needs the system's print service (CUPS).

## What was done in PisiWorks

PisiWorks was forked from Emberframe 0.1. Emberframe gave us a sound start: the window layout, the tool palette and panels, the icons, the object model with layers, pages, states and symbols, the basic drawing and bitmap tools, path operations, Auto Shapes and export all come from it, and about 88% of that code is still in place.

On top of it we added, piece by piece, everything a teacher who still teaches Fireworks would need. Emberframe's menus had 42 commands greyed out; PisiWorks has none.

| | Emberframe 0.1 | PisiWorks |
|---|---|---|
| Source code | 12,286 lines in 16 files | 37,312 lines in 35 files (two programs) |
| Automated tests | 141 | 318 |
| Menu commands that do nothing (greyed out) | 42 | 0 |
| Interface language | English | Turkish and English |

Written for PisiWorks:

| Area | Added in PisiWorks |
|---|---|
| **Linux and Pardus** | Builds and runs on Linux, a fix for the crash when typing Turkish letters through an input method, a font chosen from those installed, a plain message when the graphics driver is missing, a `.deb` package, an application icon and the `.pisi` file type |
| **Turkish** | The whole interface in Turkish, with the terms of the Turkish edition of Fireworks, a language menu, tests that fail when a text has no translation, and a user guide in Turkish |
| **Tools** | 21 new tools: Select Behind, Oval Marquee, Polygon Lasso, Sharpen, Dodge, Burn, Smudge, Replace Color, Red Eye Removal, Gradient, Vector Path, Redraw Path, Polygon, Freeform, Reshape Area, Path Scrubber, 9-Slice Scaling, and polygon and circle Hotspot and Slice tools; tool groups and tool options |
| **Vectors** | Bezier curves, full point and handle editing with the Pen and Subselection tools, and curve fitting after path operations |
| **Transforming** | Scale, Skew and Distort with a movable center point, and Numeric Transform |
| **Fills and appearance** | Gradient fills of 12 kinds, pattern and web-dither fills, gradient handles on the canvas, edge and texture; 45 blend modes; masks with all their commands |
| **Filters** | 29 live filters including Levels and Curves; Photoshop Live Effects with ten effects, all their settings and a gradient editor |
| **Bitmaps** | Moving selected pixels, Feather, Select Similar and Inverse, the marquee commands, Flatten Selection, Merge Down and Flatten Layers |
| **Text** | Fonts, bold, italic, underline, five alignments, vertical and right-to-left text, text on a path, and spell checking with the system's dictionaries |
| **Layout** | Guides, smart guides, slice guides, an adjustable grid with Snap to Grid, and Subselect and Superselect for groups |
| **Symbols and animation** | Editing a symbol in place, New Symbol, button symbols with four states (answering the mouse in Preview and in the exported HTML), Animate Selection and an animation path that can be dragged |
| **Files** | Reading the layers of Fireworks PNG files, New from Template and Save as Template, Batch Process, Image Preview, and selective JPEG quality with a JPEG encoder of its own |
| **Commands** | Saving the steps of the History panel as commands and playing them again, command files, and ready-made commands (Add Arrowheads, Add Picture Frame, Add Shadow, Auto Vector Mask, Twist and Fade, Grayscale, Sepia, Distribute to Layers and others) |
| **Printing** | Page Setup, page ranges, tiling large pages over sheets, sending to a printer or saving a PDF |
| **Settings** | Preferences, and keyboard shortcuts that can be reassigned for every menu command |
| **Animated GIF** | Opening an animated GIF as states, the number of times it plays, Tween Instances for the frames in between, and Make a GIF in the right-hand menu, which turns picture files into a GIF in one step |
| **Examples** | An Examples menu at the right end of the menu bar, apart from the Fireworks menus: seven ready documents to open, look into and change (web banner, navigation buttons, animation, school poster, gradients, filters, web page mockup) |

The figures are from November 2026 and will drift as work goes on.

## PisiAnimasyon: the animation program (first stage)

A second program is built from the same repository: **PisiAnimasyon**, an animation program in the manner of Adobe Flash, which the same schools still teach. It uses all of PisiWorks' drawing tools, filters and symbols, and adds a timeline under the stage.

```bash
cargo run --release --bin pisianimasyon
```

Working at this stage:

- The timeline: layers, frames, keyframes and blank keyframes (F5, F6, F7; Shift+F5 and Shift+F6 to remove).
- Motion tweens: between two keyframes, place, size, turn and opacity change step by step. Changing an object on a frame inside a tween makes a keyframe of that frame, as in Flash.
- Shape tweens: the shapes of a keyframe change, outline and color, into those of the next keyframe.
- Shape hints (Ctrl+Shift+H): lettered marks, dragged on the stage, that say which point of a shape is to become which.
- Guide layers: the tweened objects of the layers tied to one travel along its path, turning to face the way it runs if asked. Mask layers: the layers tied to one show only where the mask has something drawn. Several layers can be tied to one guide or mask. Guides are not drawn when publishing; masks are applied in Preview, when publishing and, as in Flash, on the stage once their layers are locked.
- Easing (from −100 to 100) and, for a motion tween, a number of turns.
- Movie clip symbols: symbols with a timeline of their own. A double-click on a copy opens the clip's own page; in the published film it plays its own frames even while the scene stands still.
- Button symbols and ready-made actions: Stop, Play, Go to Frame and Play, Go to Frame and Stop, Go to Scene, Go to Address. An action is given to a keyframe or to a button; no code is written.
- Scenes: every page in the Scenes panel is a scene, and the film plays them one after another.
- Sound: an MP3, OGG or WAV file is imported and started on a keyframe.
- Test Movie (Ctrl+Enter) publishes the film and opens it in the browser.
- Right-click menus: in the timeline, on a frame, the tween, frame and keyframe commands, and on a layer's name the layer commands; on the stage, on an object or an empty spot, the editing, arranging, transforming and symbol commands. PisiWorks' canvas has the same menu.
- Playing, stepping frame by frame, the frame rate and onion skinning.
- Playing once to the end as Flash does, with Loop Playback as an option; boxes to go straight to a frame or a second, second marks on the ruler, and a timeline that follows the playhead through long films.
- A transformation point, as in Flash: moved with the Scale tool, it stays with the object, and a motion tween turns the object around it, so an arm turns at the shoulder.
- **Behaviors**, PisiAnimasyon's own: nineteen ready behaviors given to an object with one click, with no keyframes. Spin, bounce, go back and forth, keep going, shake, grow and shrink, blink, fade, disappear after a while; and nine for the published film: follow the mouse, walk with the arrow keys, fall and jump, drag, bounce off the edges, roll like a wheel, camera follows me, slide like scenery (a side-scrolling camera with parallax), and solid (ground, platforms and walls).
- **Rules**, PisiAnimasyon's own: games with no code. A rule is an event and steps, both picked from lists and shown as a sentence: "When Apple touches Basket: increase Score by 1". Events: start, a key, a click, two objects touching, the edge of the stage, every so many seconds, a number reaching a value. Rules can have a condition ("if Score is more than 2"), make copies of an object (bullets), remove objects, make them faster, turn them, resize them, make them see-through, give them a behavior or take one away, send one object after another, attach one object to another, and set an object going so that it gathers speed and slows down (with the key-released event: a car that coasts to a stop, its wheels turning as far as it goes). Numbers such as a score are shown by any text with `{Score}` in it.
- **The language**: rules can be written too. **Rules > Rules as Text** opens the game as sentences (`When Apple touches Basket:`); a line is read as it is typed, and one that cannot be read turns red with the reason. The writing has more than the lists: sums (`Score * 2`, `Ship.x + 20`, `random(1, 6)`), several conditions joined by `and`, `otherwise`, `wait`, loops (`repeat 3 times:`), nested `if` blocks and commands done by name (`The command Burst:`). Sentences and names are offered under the caret as you write; Tab takes one. With the program in Turkish the same rules read in Turkish sentences.
- The guide is inside the program: **Help > User Guide** (F1) opens it in the browser, with no network needed. In PisiAnimasyon it opens where the lessons on animation and games begin.
- **Pisi: Iron Claw**, the showcase: a run-and-gun of some length with a robot leopard, robot rats that shoot back, drones, three stages and a giant rat at the end. Twenty-eight rules and not a line of code; open it from the Examples menu and read every one.

<p align="center"><img src="docs/images/pisi-demir-pence.png" alt="Pisi: Iron Claw" width="720"></p>

- Fifteen more ready films in the Examples menu: a bouncing ball, changing shapes, along a path, a spotlight, turning gears, one after another, a stick-figure fight, behaviors, five games (catch the apples, hit the targets, a platform game, a space shooter, a quiz) with drawn characters, moving scenery, bursts and sounds made in the program, driving a car, and two scenes with buttons. The guide has each game as a lesson. The guide has the fight as a lesson, step by step.
- Exchange with PisiWorks: **File > Open in PisiAnimasyon** in PisiWorks opens the document in the animation program, and **File > Open Frame in PisiWorks** in PisiAnimasyon opens the frame on show in the design program, each as a copy. **File > Import** in either program brings in a document of the other, with its objects and symbols.
- `.sahne` documents. PisiWorks opens `.sahne` files and PisiAnimasyon opens `.pisi` files; artwork passes between the two as it is.
- **Control > Publish as HTML5** writes the animation as a single HTML file that plays in a browser. Each frame is in the page as a picture, drawn as the program draws it.

**Site and book:** [muratsapmaz.github.io/PisiAnimasyon](https://muratsapmaz.github.io/PisiAnimasyon/) (its repository: [MuratSAPMAZ/PisiAnimasyon](https://github.com/MuratSAPMAZ/PisiAnimasyon); its pages are built from `docs/site/` and `docs/kitap/` here by `packaging/site.py`).

**The PisiAnimasyon book.** A book for students and teachers comes with PisiAnimasyon, in Turkish, with pictures and in lessons: 31 chapters in six parts (getting started, motion, the rules of a game, six games as lessons, writing the rules, drawing and characters) and appendices (every sentence, shortcuts and common questions, a term's plan for the teacher). Each chapter goes step by step with pictures of the program, has a part for when one is stuck and things to try, and a note for the teacher. It comes two ways: with pictures only, fit for printing, and with the same examples as films that really play in the page (twenty of them; a click gives one the keyboard). A link at the top of every page leads to the other. In PisiAnimasyon, **Help > User Guide** (F1) opens the live one in the browser when the interface is in Turkish. Its sources are in `docs/kitap/`; `packaging/kitap.py` builds the pages.

The window is laid out as Flash's classic workspace is: the timeline over the stage, the tools on the left, Properties at the bottom; layers are in the timeline. Enter plays. The `.deb` package installs both programs.

Known differences from Flash: there is no ActionScript; ready-made actions take its place. `.fla` and `.swf` files are neither opened nor written. In the published page every frame, and every frame of a clip, is a picture. Clips inside clips play by their own frames too; only a button placed inside a clip cannot be clicked in the published film (buttons on the stage can). While playing inside the program (Enter), frame actions are not run and clips show their first frame, as in Flash; the whole film is seen with Test Movie.

The Turkish terms are our own rendering, for want of a Turkish Flash installation to take them from, and may differ from the words in teachers' notes. They are all in one translation table and will be corrected if the right ones are reported.

## Not in this version

- Fireworks' JavaScript (JSF) scripts and extensions: Run Script plays only PisiWorks' own command files (one menu command on each line). The dialogs of the ready-made commands are not copies of Fireworks' own: Add Arrowheads offers five heads and Auto Vector Mask seven masks. Adobe's templates are not there either; New from Template lists our own and the ones you save.
- Paths, text and groups in Fireworks PNG files. Layers, states, bitmap objects and the Drop Shadow effect are read; a file that has a path, text or group opens as its flat picture. Adobe never published the format, so the reader was worked out from a few files written by Fireworks 8 and has not been tried on files from other versions. Fireworks PNG is not written.
- Photoshop's own pattern and gradient libraries in Photoshop Live Effects.
- A ready-made package to download. For now the `.deb` is built from source.

The program has not been tested side by side with Adobe Fireworks itself; the pixel results of filters and blend modes may differ from those of Fireworks. The one exception: a Drop Shadow (softness 1) in a file written by Fireworks 8 was compared with our drawing of it, and 287 of 289 pixels came out the same. Reports of differences and bugs are welcome.

## Who we are, license and thanks

**PisiWorks** is developed and maintained by **Murat SAPMAZ**: [muratsapmaz.web.app](https://muratsapmaz.web.app/) · [github.com/MuratSAPMAZ/PisiWorks](https://github.com/MuratSAPMAZ/PisiWorks). Bug reports, suggestions and contributions are welcome there. It is an independent project, not affiliated with TÜBİTAK ULAKBİM, which develops Pardus, nor with Adobe.

**Thanks.** PisiWorks stands on Emberframe, which **Med Amine Belbachir** wrote and opened to everyone under the MIT license. We thank him for his work and his generosity:

- Website: [medamineweb.com](https://medamineweb.com/)
- Emberframe: [github.com/gregstein/emberframe](https://github.com/gregstein/emberframe) · [project page](https://medamineweb.com/en/emberframe/)
- GitHub: [@gregstein](https://github.com/gregstein)

**License.** PisiWorks and PisiAnimasyon are free software under the [GNU General Public License, version 3 or later](COPYING): anyone may use, study, change and pass them on, and whoever passes on a changed version must give its source under the same license. The code that came from Emberframe stays under its author's [MIT license](LICENSES/MIT.txt). Three additional terms go with the license ([NOTICE](NOTICE)): the developer and the source named in About are kept; a modified version says plainly that it is one; and **the names PisiWorks and PisiAnimasyon, and their icons, may not be used for a modified version**, which goes under a name of its own. **The films and pages you make with the programs are yours**: you may pass them on, with the player script written into them, under any terms ([NOTICE](NOTICE)). Where the code comes from, the licenses of the libraries compiled into it ([list](docs/THIRD-PARTY.md)) and how it stands toward Adobe are set out in the [license and copyright statement](docs/lisans-ve-telif.md) (in Turkish).

Adobe, Fireworks, Flash, Photoshop and ActionScript are trademarks of Adobe Inc. PisiWorks and PisiAnimasyon contain no code, artwork or documents of Adobe's; they are not affiliated with Adobe and are not endorsed by it. These names are used only to say what the programs resemble and which files they can open.
