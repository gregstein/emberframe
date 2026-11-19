# Third-party libraries

PisiWorks is built from 517 libraries, all of them free software. This list is made by
`packaging/third-party.py` from what Cargo compiles into the Linux program; the libraries used
only on other systems, for building or for tests are not in it. Where a library offers a choice
of licenses, PisiWorks takes it under the most permissive one (MIT or Apache-2.0).

## By license

| License | Libraries |
|---|---|
| MIT OR Apache-2.0 | 227 |
| MIT | 125 |
| Apache-2.0 OR MIT | 64 |
| Apache-2.0 | 22 |
| Unicode-3.0 | 18 |
| Unlicense OR MIT | 8 |
| BSD-3-Clause | 8 |
| MIT OR Apache-2.0 OR Zlib | 5 |
| BSD-2-Clause | 4 |
| Zlib | 4 |
| Apache-2.0 OR ISC OR MIT | 4 |
| Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT | 4 |
| Zlib OR Apache-2.0 OR MIT | 3 |
| ISC | 3 |
| CC0-1.0 | 2 |
| MIT OR Zlib OR Apache-2.0 | 2 |
| BSD-3-Clause OR Apache-2.0 | 2 |
| (MIT OR Apache-2.0) AND Unicode-3.0 | 2 |
| BSD-2-Clause OR Apache-2.0 OR MIT | 2 |
| 0BSD OR MIT OR Apache-2.0 | 1 |
| CC0-1.0 OR MIT-0 OR Apache-2.0 | 1 |
| (Apache-2.0 OR MIT) AND BSD-3-Clause | 1 |
| CC0-1.0 OR Apache-2.0 | 1 |
| MPL-2.0 | 1 |
| Apache-2.0 AND ISC | 1 |
| Apache-2.0 OR BSL-1.0 | 1 |
| Apache-2.0 OR GPL-2.0-only | 1 |

## Every library

| Library | Version | License | Source |
|---|---|---|---|
| ab_glyph | 0.2.32 | Apache-2.0 | https://github.com/alexheretic/ab-glyph |
| ab_glyph_rasterizer | 0.1.10 | Apache-2.0 | https://github.com/alexheretic/ab-glyph |
| adler2 | 2.0.1 | 0BSD OR MIT OR Apache-2.0 | https://github.com/oyvindln/adler2 |
| aes | 0.8.4 | MIT OR Apache-2.0 | https://github.com/RustCrypto/block-ciphers |
| ahash | 0.8.12 | MIT OR Apache-2.0 | https://github.com/tkaitchuck/ahash |
| aho-corasick | 1.1.5 | Unlicense OR MIT | https://github.com/BurntSushi/aho-corasick |
| aligned | 0.4.3 | MIT OR Apache-2.0 | https://github.com/rust-embedded-community/aligned |
| aligned-vec | 0.6.4 | MIT | https://github.com/sarah-ek/aligned-vec/ |
| anyhow | 1.0.104 | MIT OR Apache-2.0 | https://github.com/dtolnay/anyhow |
| arg_enum_proc_macro | 0.3.4 | MIT | https://github.com/lu-zero/arg_enum_proc_macro |
| arrayref | 0.3.9 | BSD-2-Clause | https://github.com/droundy/arrayref |
| arrayvec | 0.7.8 | MIT OR Apache-2.0 | https://github.com/bluss/arrayvec |
| as-raw-xcb-connection | 1.0.1 | MIT OR Apache-2.0 | https://github.com/psychon/as-raw-xcb-connection |
| as-slice | 0.2.1 | MIT OR Apache-2.0 | https://github.com/japaric/as-slice |
| ash | 0.38.0+1.3.281 | MIT OR Apache-2.0 | https://github.com/ash-rs/ash |
| ash-window | 0.13.0 | MIT OR Apache-2.0 | https://github.com/ash-rs/ash |
| ashpd | 0.11.1 | MIT | https://github.com/bilelmoussaoui/ashpd |
| ashpd | 0.12.3 | MIT | https://github.com/bilelmoussaoui/ashpd |
| async-broadcast | 0.7.2 | MIT OR Apache-2.0 | https://github.com/smol-rs/async-broadcast |
| async-channel | 1.9.0 | Apache-2.0 OR MIT | https://github.com/smol-rs/async-channel |
| async-channel | 2.5.0 | Apache-2.0 OR MIT | https://github.com/smol-rs/async-channel |
| async-compression | 0.4.52 | MIT OR Apache-2.0 | https://github.com/Nullus157/async-compression |
| async-executor | 1.14.1 | Apache-2.0 OR MIT | https://github.com/smol-rs/async-executor |
| async-fs | 2.2.0 | Apache-2.0 OR MIT | https://github.com/smol-rs/async-fs |
| async-global-executor | 2.4.1 | Apache-2.0 OR MIT | https://github.com/Keruspe/async-global-executor |
| async-io | 2.6.0 | Apache-2.0 OR MIT | https://github.com/smol-rs/async-io |
| async-lock | 3.5.0 | Apache-2.0 OR MIT | https://github.com/smol-rs/async-lock |
| async-net | 2.0.0 | Apache-2.0 OR MIT | https://github.com/smol-rs/async-net |
| async-process | 2.5.0 | Apache-2.0 OR MIT | https://github.com/smol-rs/async-process |
| async-recursion | 1.2.0 | MIT OR Apache-2.0 | https://github.com/dcchut/async-recursion |
| async-signal | 0.2.14 | Apache-2.0 OR MIT | https://github.com/smol-rs/async-signal |
| async-std | 1.13.2 | Apache-2.0 OR MIT | https://github.com/async-rs/async-std |
| async-task | 4.7.1 | Apache-2.0 OR MIT | https://github.com/smol-rs/async-task |
| async-trait | 0.1.92 | MIT OR Apache-2.0 | https://github.com/dtolnay/async-trait |
| async_zip | 0.0.17 | MIT | https://github.com/Majored/rs-async-zip |
| atomic | 0.5.3 | Apache-2.0/MIT | https://github.com/Amanieu/atomic-rs |
| atomic-waker | 1.1.2 | Apache-2.0 OR MIT | https://github.com/smol-rs/atomic-waker |
| av-scenechange | 0.14.1 | MIT | https://github.com/rust-av/av-scenechange |
| av1-grain | 0.2.5 | BSD-2-Clause | https://github.com/rust-av/av1-grain |
| avif-serialize | 0.8.9 | BSD-3-Clause | https://github.com/kornelski/avif-serialize |
| base64 | 0.22.1 | MIT OR Apache-2.0 | https://github.com/marshallpierce/rust-base64 |
| bit-set | 0.8.0 | Apache-2.0 OR MIT | https://github.com/contain-rs/bit-set |
| bit-vec | 0.8.0 | Apache-2.0 OR MIT | https://github.com/contain-rs/bit-vec |
| bit_field | 0.10.3 | Apache-2.0/MIT | https://github.com/phil-opp/rust-bit-field |
| bitflags | 1.3.2 | MIT/Apache-2.0 | https://github.com/bitflags/bitflags |
| bitflags | 2.13.2 | MIT OR Apache-2.0 | https://github.com/bitflags/bitflags |
| bitstream-io | 4.10.0 | MIT/Apache-2.0 | https://github.com/tuffy/bitstream-io |
| blade-graphics | 0.7.1 | MIT | https://github.com/kvark/blade |
| blade-macros | 0.3.0 | MIT | https://github.com/kvark/blade |
| blade-util | 0.3.0 | MIT | https://github.com/kvark/blade |
| block-buffer | 0.10.4 | MIT OR Apache-2.0 | https://github.com/RustCrypto/utils |
| block-buffer | 0.12.1 | MIT OR Apache-2.0 | https://github.com/RustCrypto/utils |
| block-padding | 0.3.3 | MIT OR Apache-2.0 | https://github.com/RustCrypto/utils |
| blocking | 1.7.0 | Apache-2.0 OR MIT | https://github.com/smol-rs/blocking |
| bstr | 1.13.1 | MIT OR Apache-2.0 | https://github.com/BurntSushi/bstr |
| bytemuck | 1.25.2 | Zlib OR Apache-2.0 OR MIT | https://github.com/Lokathor/bytemuck |
| bytemuck_derive | 1.12.1 | Zlib OR Apache-2.0 OR MIT | https://github.com/Lokathor/bytemuck |
| byteorder | 1.5.0 | Unlicense OR MIT | https://github.com/BurntSushi/byteorder |
| byteorder-lite | 0.1.0 | Unlicense OR MIT | https://github.com/image-rs/byteorder-lite |
| bytes | 1.12.1 | MIT | https://github.com/tokio-rs/bytes |
| calloop | 0.13.0 | MIT | https://github.com/Smithay/calloop |
| calloop-wayland-source | 0.3.0 | MIT | https://github.com/smithay/calloop-wayland-source |
| cbc | 0.1.2 | MIT OR Apache-2.0 | https://github.com/RustCrypto/block-modes |
| cfg-if | 1.0.5 | MIT OR Apache-2.0 | https://github.com/rust-lang/cfg-if |
| cipher | 0.4.4 | MIT OR Apache-2.0 | https://github.com/RustCrypto/traits |
| codespan-reporting | 0.12.0 | Apache-2.0 | https://github.com/brendanzab/codespan |
| color_quant | 1.1.0 | MIT | https://github.com/image-rs/color_quant.git |
| command-fds | 0.3.3 | Apache-2.0 | https://github.com/google/command-fds/ |
| compression-codecs | 0.4.47 | MIT OR Apache-2.0 | https://github.com/Nullus157/async-compression |
| compression-core | 0.4.34 | MIT OR Apache-2.0 | https://github.com/Nullus157/async-compression |
| concurrent-queue | 2.5.0 | Apache-2.0 OR MIT | https://github.com/smol-rs/concurrent-queue |
| const-oid | 0.10.2 | Apache-2.0 OR MIT | https://github.com/RustCrypto/formats |
| const-random | 0.1.18 | MIT OR Apache-2.0 | https://github.com/tkaitchuck/constrandom |
| const-random-macro | 0.1.16 | MIT OR Apache-2.0 | https://github.com/tkaitchuck/constrandom |
| convert_case | 0.4.0 | MIT | https://github.com/rutrum/convert-case |
| core_detect | 1.0.0 | MIT/Apache-2.0 | https://github.com/thomcc/core_detect |
| core_maths | 0.1.1 | MIT | https://github.com/robertbastian/core_maths |
| cosmic-text | 0.14.2 | MIT OR Apache-2.0 | https://github.com/pop-os/cosmic-text |
| cpufeatures | 0.2.17 | MIT OR Apache-2.0 | https://github.com/RustCrypto/utils |
| cpufeatures | 0.3.1 | MIT OR Apache-2.0 | https://github.com/RustCrypto/utils |
| crc32fast | 1.5.4 | MIT OR Apache-2.0 | https://github.com/srijs/rust-crc32fast |
| crossbeam-deque | 0.8.8 | MIT OR Apache-2.0 | https://github.com/crossbeam-rs/crossbeam |
| crossbeam-epoch | 0.9.21 | MIT OR Apache-2.0 | https://github.com/crossbeam-rs/crossbeam |
| crossbeam-queue | 0.3.14 | MIT OR Apache-2.0 | https://github.com/crossbeam-rs/crossbeam |
| crossbeam-utils | 0.8.23 | MIT OR Apache-2.0 | https://github.com/crossbeam-rs/crossbeam |
| crunchy | 0.2.4 | MIT | https://github.com/eira-fransham/crunchy |
| crypto-common | 0.1.7 | MIT OR Apache-2.0 | https://github.com/RustCrypto/traits |
| crypto-common | 0.2.2 | MIT OR Apache-2.0 | https://github.com/RustCrypto/traits |
| ctor | 0.4.3 | Apache-2.0 OR MIT | https://github.com/mmastrac/rust-ctor |
| ctor-proc-macro | 0.0.6 | Apache-2.0 OR MIT | https://github.com/mmastrac/rust-ctor |
| data-url | 0.3.2 | MIT OR Apache-2.0 | https://github.com/servo/rust-url |
| deflate64 | 0.1.12 | MIT | https://github.com/anatawa12/deflate64-rs |
| derive_more | 0.99.20 | MIT | https://github.com/JelteF/derive_more |
| digest | 0.10.7 | MIT OR Apache-2.0 | https://github.com/RustCrypto/traits |
| digest | 0.11.3 | MIT OR Apache-2.0 | https://github.com/RustCrypto/traits |
| dirs | 4.0.0 | MIT OR Apache-2.0 | https://github.com/soc/dirs-rs |
| dirs | 5.0.1 | MIT OR Apache-2.0 | https://github.com/soc/dirs-rs |
| dirs-sys | 0.3.7 | MIT OR Apache-2.0 | https://github.com/dirs-dev/dirs-sys-rs |
| dirs-sys | 0.4.1 | MIT OR Apache-2.0 | https://github.com/dirs-dev/dirs-sys-rs |
| displaydoc | 0.2.7 | MIT OR Apache-2.0 | https://github.com/yaahc/displaydoc |
| dlib | 0.5.3 | MIT | https://github.com/elinorbgr/dlib |
| downcast-rs | 1.2.1 | MIT/Apache-2.0 | https://github.com/marcianx/downcast-rs |
| dtor | 0.0.6 | Apache-2.0 OR MIT | https://github.com/mmastrac/rust-ctor |
| dtor-proc-macro | 0.0.5 | Apache-2.0 OR MIT | https://github.com/mmastrac/rust-ctor |
| dunce | 1.0.5 | CC0-1.0 OR MIT-0 OR Apache-2.0 | https://gitlab.com/kornelski/dunce |
| dyn-clone | 1.0.20 | MIT OR Apache-2.0 | https://github.com/dtolnay/dyn-clone |
| either | 1.19.0 | MIT OR Apache-2.0 | https://github.com/rayon-rs/either |
| encoding_rs | 0.8.44 | (Apache-2.0 OR MIT) AND BSD-3-Clause | https://github.com/hsivonen/encoding_rs |
| endi | 1.1.1 | MIT | https://github.com/zeenix/endi |
| enumflags2 | 0.7.12 | MIT OR Apache-2.0 | https://github.com/meithecatte/enumflags2 |
| enumflags2_derive | 0.7.12 | MIT OR Apache-2.0 | https://github.com/meithecatte/enumflags2 |
| equator | 0.4.2 | MIT | https://github.com/sarah-ek/equator/ |
| equator-macro | 0.4.2 | MIT | https://github.com/sarah-ek/equator/ |
| equivalent | 1.0.3 | Apache-2.0 OR MIT | https://github.com/indexmap-rs/equivalent |
| erased-serde | 0.4.11 | MIT OR Apache-2.0 | https://github.com/dtolnay/erased-serde |
| errno | 0.3.14 | MIT OR Apache-2.0 | https://github.com/lambda-fairy/rust-errno |
| etagere | 0.2.15 | MIT/Apache-2.0 | https://github.com/nical/etagere |
| euclid | 0.22.14 | MIT OR Apache-2.0 | https://github.com/servo/euclid |
| event-listener | 2.5.3 | Apache-2.0 OR MIT | https://github.com/smol-rs/event-listener |
| event-listener | 5.4.2 | Apache-2.0 OR MIT | https://github.com/smol-rs/event-listener |
| event-listener-strategy | 0.5.4 | Apache-2.0 OR MIT | https://github.com/smol-rs/event-listener-strategy |
| exr | 1.74.2 | BSD-3-Clause | https://github.com/johannesvollmer/exrs |
| fastrand | 1.9.0 | Apache-2.0 OR MIT | https://github.com/smol-rs/fastrand |
| fastrand | 2.5.0 | Apache-2.0 OR MIT | https://github.com/smol-rs/fastrand |
| fax | 0.2.7 | MIT | https://github.com/pdf-rs/fax |
| fdeflate | 0.3.7 | MIT OR Apache-2.0 | https://github.com/image-rs/fdeflate |
| filedescriptor | 0.8.3 | MIT | https://github.com/wezterm/wezterm |
| filetime | 0.2.29 | MIT/Apache-2.0 | https://github.com/alexcrichton/filetime |
| flate2 | 1.1.11 | MIT OR Apache-2.0 | https://github.com/rust-lang/flate2-rs |
| float-cmp | 0.9.0 | MIT | https://github.com/mikedilger/float-cmp |
| float-ord | 0.3.2 | MIT / Apache-2.0 | https://github.com/notriddle/rust-float-ord |
| float_next_after | 1.0.0 | MIT | https://gitlab.com/bronsonbdevost/next_afterf |
| flume | 0.11.1 | Apache-2.0/MIT | https://github.com/zesterer/flume |
| fnv | 1.0.7 | Apache-2.0 / MIT | https://github.com/servo/rust-fnv |
| foldhash | 0.1.5 | Zlib | https://github.com/orlp/foldhash |
| foldhash | 0.2.0 | Zlib | https://github.com/orlp/foldhash |
| font-types | 0.12.7 | MIT OR Apache-2.0 | https://github.com/googlefonts/fontations |
| fontconfig-parser | 0.5.8 | MIT | https://github.com/Riey/fontconfig-parser |
| fontdb | 0.16.2 | MIT | https://github.com/RazrFalcon/fontdb |
| fontdb | 0.23.0 | MIT | https://github.com/RazrFalcon/fontdb |
| form_urlencoded | 1.2.2 | MIT OR Apache-2.0 | https://github.com/servo/rust-url |
| freetype-sys | 0.20.1 | MIT | https://github.com/PistonDevelopers/freetype-sys.git |
| futures | 0.3.34 | MIT OR Apache-2.0 | https://github.com/rust-lang/futures-rs |
| futures-channel | 0.3.34 | MIT OR Apache-2.0 | https://github.com/rust-lang/futures-rs |
| futures-core | 0.3.34 | MIT OR Apache-2.0 | https://github.com/rust-lang/futures-rs |
| futures-executor | 0.3.34 | MIT OR Apache-2.0 | https://github.com/rust-lang/futures-rs |
| futures-io | 0.3.34 | MIT OR Apache-2.0 | https://github.com/rust-lang/futures-rs |
| futures-lite | 1.13.0 | Apache-2.0 OR MIT | https://github.com/smol-rs/futures-lite |
| futures-lite | 2.6.1 | Apache-2.0 OR MIT | https://github.com/smol-rs/futures-lite |
| futures-macro | 0.3.34 | MIT OR Apache-2.0 | https://github.com/rust-lang/futures-rs |
| futures-sink | 0.3.34 | MIT OR Apache-2.0 | https://github.com/rust-lang/futures-rs |
| futures-task | 0.3.34 | MIT OR Apache-2.0 | https://github.com/rust-lang/futures-rs |
| futures-util | 0.3.34 | MIT OR Apache-2.0 | https://github.com/rust-lang/futures-rs |
| generic-array | 0.14.7 | MIT | https://github.com/fizyk20/generic-array.git |
| gethostname | 1.1.0 | Apache-2.0 | https://codeberg.org/swsnr/gethostname.rs.git |
| getrandom | 0.2.17 | MIT OR Apache-2.0 | https://github.com/rust-random/getrandom |
| getrandom | 0.3.4 | MIT OR Apache-2.0 | https://github.com/rust-random/getrandom |
| getrandom | 0.4.3 | MIT OR Apache-2.0 | https://github.com/rust-random/getrandom |
| gif | 0.14.2 | MIT OR Apache-2.0 | https://github.com/image-rs/image-gif |
| globset | 0.4.21 | Unlicense OR MIT | https://github.com/BurntSushi/ripgrep/tree/master/crates/globset |
| gpu-alloc | 0.6.2 | MIT OR Apache-2.0 | https://github.com/zakarumych/gpu-alloc |
| gpu-alloc-ash | 0.7.1 | MIT OR Apache-2.0 | https://github.com/zakarumych/gpu-alloc |
| gpu-alloc-types | 0.3.1 | MIT OR Apache-2.0 | https://github.com/zakarumych/gpu-alloc |
| gpui | 0.2.2 | Apache-2.0 | https://github.com/zed-industries/zed |
| gpui-macros | 0.2.2 | Apache-2.0 |  |
| gpui_collections | 0.2.2 | Apache-2.0 |  |
| gpui_derive_refineable | 0.2.2 | Apache-2.0 |  |
| gpui_http_client | 0.2.2 | Apache-2.0 |  |
| gpui_perf | 0.2.2 | Apache-2.0 |  |
| gpui_refineable | 0.2.2 | Apache-2.0 |  |
| gpui_semantic_version | 0.2.2 | Apache-2.0 |  |
| gpui_sum_tree | 0.2.2 | Apache-2.0 |  |
| gpui_util | 0.2.2 | Apache-2.0 |  |
| gpui_util_macros | 0.2.2 | Apache-2.0 |  |
| grid | 0.18.0 | MIT | https://github.com/becheran/grid |
| h2 | 0.4.20 | MIT | https://github.com/hyperium/h2 |
| half | 2.7.1 | MIT OR Apache-2.0 | https://github.com/VoidStarKat/half-rs |
| hashbrown | 0.14.5 | MIT OR Apache-2.0 | https://github.com/rust-lang/hashbrown |
| hashbrown | 0.15.5 | MIT OR Apache-2.0 | https://github.com/rust-lang/hashbrown |
| hashbrown | 0.17.1 | MIT OR Apache-2.0 | https://github.com/rust-lang/hashbrown |
| heck | 0.5.0 | MIT OR Apache-2.0 | https://github.com/withoutboats/heck |
| hex | 0.4.3 | MIT OR Apache-2.0 | https://github.com/KokaKiwi/rust-hex |
| hexf-parse | 0.2.1 | CC0-1.0 | https://github.com/lifthrasiir/hexf |
| hidden-trait | 0.1.2 | MIT | https://github.com/kvark/hidden-trait |
| hkdf | 0.12.4 | MIT OR Apache-2.0 | https://github.com/RustCrypto/KDFs/ |
| hmac | 0.12.1 | MIT OR Apache-2.0 | https://github.com/RustCrypto/MACs |
| home | 0.5.12 | MIT OR Apache-2.0 | https://github.com/rust-lang/cargo |
| http | 1.5.0 | MIT OR Apache-2.0 | https://github.com/hyperium/http |
| http-body | 1.1.0 | MIT | https://github.com/hyperium/http-body |
| http-body-util | 0.1.5 | MIT | https://github.com/hyperium/http-body |
| httparse | 1.10.1 | MIT OR Apache-2.0 | https://github.com/seanmonstar/httparse |
| hybrid-array | 0.4.15 | MIT OR Apache-2.0 | https://github.com/RustCrypto/hybrid-array |
| hyper | 1.12.0 | MIT | https://github.com/hyperium/hyper |
| hyper-rustls | 0.27.10 | Apache-2.0 OR ISC OR MIT | https://github.com/rustls/hyper-rustls |
| hyper-util | 0.1.21 | MIT | https://github.com/hyperium/hyper-util |
| i_float | 5.0.0 | MIT | https://github.com/iShape-Rust/iFloat |
| i_key_sort | 0.12.0 | MIT | https://github.com/iShape-Rust/iKeySort |
| i_overlay | 9.1.0 | MIT OR Apache-2.0 | https://github.com/iShape-Rust/iOverlay |
| i_shape | 5.0.0 | MIT | https://github.com/iShape-Rust/i_shape |
| i_tree | 0.20.0 | MIT | https://github.com/iShape-Rust/iTree |
| icu_collections | 2.4.0 | Unicode-3.0 | https://github.com/unicode-org/icu4x |
| icu_locale_core | 2.4.0 | Unicode-3.0 | https://github.com/unicode-org/icu4x |
| icu_normalizer | 2.4.0 | Unicode-3.0 | https://github.com/unicode-org/icu4x |
| icu_normalizer_data | 2.4.0 | Unicode-3.0 | https://github.com/unicode-org/icu4x |
| icu_properties | 2.4.0 | Unicode-3.0 | https://github.com/unicode-org/icu4x |
| icu_properties_data | 2.4.0 | Unicode-3.0 | https://github.com/unicode-org/icu4x |
| icu_provider | 2.4.0 | Unicode-3.0 | https://github.com/unicode-org/icu4x |
| idna | 1.1.0 | MIT OR Apache-2.0 | https://github.com/servo/rust-url/ |
| idna_adapter | 1.2.2 | Apache-2.0 OR MIT | https://github.com/hsivonen/idna_adapter |
| image | 0.25.10 | MIT OR Apache-2.0 | https://github.com/image-rs/image |
| image-webp | 0.2.4 | MIT OR Apache-2.0 | https://github.com/image-rs/image-webp |
| imagesize | 0.13.0 | MIT | https://github.com/Roughsketch/imagesize |
| imgref | 1.12.3 | CC0-1.0 OR Apache-2.0 | https://github.com/kornelski/imgref |
| indexmap | 2.14.2 | Apache-2.0 OR MIT | https://github.com/indexmap-rs/indexmap |
| inout | 0.1.4 | MIT OR Apache-2.0 | https://github.com/RustCrypto/utils |
| inventory | 0.3.25 | MIT OR Apache-2.0 | https://github.com/dtolnay/inventory |
| ipnet | 2.12.2 | MIT OR Apache-2.0 | https://github.com/krisprice/ipnet |
| is-docker | 0.2.0 | MIT | https://github.com/TheLarkInn/is-docker |
| is-wsl | 0.4.0 | MIT | https://github.com/TheLarkInn/is-wsl |
| itertools | 0.14.0 | MIT OR Apache-2.0 | https://github.com/rust-itertools/itertools |
| itoa | 1.0.18 | MIT OR Apache-2.0 | https://github.com/dtolnay/itoa |
| kurbo | 0.11.3 | Apache-2.0 OR MIT | https://github.com/linebender/kurbo |
| kv-log-macro | 1.0.7 | MIT OR Apache-2.0 | https://github.com/yoshuawuyts/kv-log-macro |
| lazy_static | 1.5.1 | MIT OR Apache-2.0 | https://github.com/rust-lang-nursery/lazy-static.rs |
| lebe | 0.5.3 | BSD-3-Clause | https://github.com/johannesvollmer/lebe |
| libc | 0.2.186 | MIT OR Apache-2.0 | https://github.com/rust-lang/libc |
| libloading | 0.8.9 | ISC | https://github.com/nagisa/rust_libloading/ |
| libm | 0.2.16 | MIT | https://github.com/rust-lang/compiler-builtins |
| linux-raw-sys | 0.12.1 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT | https://github.com/sunfishcode/linux-raw-sys |
| linux-raw-sys | 0.4.15 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT | https://github.com/sunfishcode/linux-raw-sys |
| litemap | 0.8.3 | Unicode-3.0 | https://github.com/unicode-org/icu4x |
| lock_api | 0.4.14 | MIT OR Apache-2.0 | https://github.com/Amanieu/parking_lot |
| log | 0.4.35 | MIT OR Apache-2.0 | https://github.com/rust-lang/log |
| loop9 | 0.1.5 | MIT | https://gitlab.com/kornelski/loop9.git |
| lyon | 1.0.19 | MIT OR Apache-2.0 | https://github.com/nical/lyon |
| lyon_algorithms | 1.0.22 | MIT OR Apache-2.0 | https://github.com/nical/lyon |
| lyon_geom | 1.0.20 | MIT OR Apache-2.0 | https://github.com/nical/lyon |
| lyon_path | 1.0.19 | MIT OR Apache-2.0 | https://github.com/nical/lyon |
| lyon_tessellation | 1.0.23 | MIT OR Apache-2.0 | https://github.com/nical/lyon |
| maybe-rayon | 0.1.1 | MIT | https://github.com/shssoichiro/maybe-rayon |
| md-5 | 0.10.6 | MIT OR Apache-2.0 | https://github.com/RustCrypto/hashes |
| memchr | 2.8.3 | Unlicense OR MIT | https://github.com/BurntSushi/memchr |
| memmap2 | 0.9.12 | MIT OR Apache-2.0 | https://github.com/RazrFalcon/memmap2-rs |
| mime | 0.3.18 | MIT OR Apache-2.0 | https://github.com/hyperium/mime |
| mime_guess | 2.0.5 | MIT | https://github.com/abonander/mime_guess |
| miniz_oxide | 0.8.9 | MIT OR Zlib OR Apache-2.0 | https://github.com/Frommi/miniz_oxide/tree/master/miniz_oxide |
| miniz_oxide | 0.9.1 | MIT OR Zlib OR Apache-2.0 | https://github.com/Frommi/miniz_oxide/tree/master/miniz_oxide |
| mint | 0.5.9 | MIT | https://github.com/kvark/mint |
| mio | 1.2.4 | MIT | https://github.com/tokio-rs/mio |
| moxcms | 0.8.1 | BSD-3-Clause OR Apache-2.0 | https://github.com/awxkee/moxcms.git |
| multiversion_no_op | 1.0.0 | Apache-2.0 OR MIT | https://github.com/hsivonen/multiversion_no_op |
| naga | 25.0.1 | MIT OR Apache-2.0 | https://github.com/gfx-rs/wgpu/tree/trunk/naga |
| nanorand | 0.7.0 | Zlib | https://github.com/Absolucy/nanorand-rs |
| new_debug_unreachable | 1.0.6 | MIT | https://github.com/mbrubeck/rust-debug-unreachable |
| nix | 0.29.0 | MIT | https://github.com/nix-rust/nix |
| nix | 0.31.3 | MIT | https://github.com/nix-rust/nix |
| no_std_io2 | 0.9.4 | Apache-2.0 OR MIT | https://github.com/wcampbell0x2a/no-std-io2 |
| nom | 8.0.0 | MIT | https://github.com/rust-bakery/nom |
| noop_proc_macro | 0.3.0 | MIT | https://github.com/lu-zero/noop_proc_macro |
| num | 0.4.3 | MIT OR Apache-2.0 | https://github.com/rust-num/num |
| num-bigint | 0.4.8 | MIT OR Apache-2.0 | https://github.com/rust-num/num-bigint |
| num-bigint-dig | 0.8.6 | MIT/Apache-2.0 | https://github.com/dignifiedquire/num-bigint |
| num-complex | 0.4.6 | MIT OR Apache-2.0 | https://github.com/rust-num/num-complex |
| num-derive | 0.4.2 | MIT OR Apache-2.0 | https://github.com/rust-num/num-derive |
| num-integer | 0.1.47 | MIT OR Apache-2.0 | https://github.com/rust-num/num-integer |
| num-iter | 0.1.46 | MIT OR Apache-2.0 | https://github.com/rust-num/num-iter |
| num-rational | 0.4.2 | MIT OR Apache-2.0 | https://github.com/rust-num/num-rational |
| num-traits | 0.2.19 | MIT OR Apache-2.0 | https://github.com/rust-num/num-traits |
| num_cpus | 1.17.0 | MIT OR Apache-2.0 | https://github.com/seanmonstar/num_cpus |
| once_cell | 1.21.4 | MIT OR Apache-2.0 | https://github.com/matklad/once_cell |
| oo7 | 0.5.0 | MIT | https://github.com/bilelmoussaoui/oo7 |
| open | 5.4.5 | MIT | https://github.com/Byron/open-rs |
| openssl-probe | 0.2.2 | MIT OR Apache-2.0 | https://github.com/rustls/openssl-probe |
| option-ext | 0.2.0 | MPL-2.0 | https://github.com/soc/option-ext.git |
| ordered-stream | 0.2.0 | MIT OR Apache-2.0 | https://github.com/danieldg/ordered-stream |
| owned_ttf_parser | 0.25.1 | Apache-2.0 | https://github.com/alexheretic/owned-ttf-parser |
| parking | 2.2.1 | Apache-2.0 OR MIT | https://github.com/smol-rs/parking |
| parking_lot | 0.12.5 | MIT OR Apache-2.0 | https://github.com/Amanieu/parking_lot |
| parking_lot_core | 0.9.12 | MIT OR Apache-2.0 | https://github.com/Amanieu/parking_lot |
| paste | 1.0.15 | MIT OR Apache-2.0 | https://github.com/dtolnay/paste |
| pastey | 0.1.1 | MIT OR Apache-2.0 | https://github.com/as1100k/pastey |
| pathfinder_geometry | 0.5.1 | MIT/Apache-2.0 | https://github.com/servo/pathfinder |
| pathfinder_simd | 0.5.6 | MIT OR Apache-2.0 | https://github.com/servo/pathfinder |
| pbkdf2 | 0.12.2 | MIT OR Apache-2.0 | https://github.com/RustCrypto/password-hashes/tree/master/pbkdf2 |
| percent-encoding | 2.3.2 | MIT OR Apache-2.0 | https://github.com/servo/rust-url/ |
| pico-args | 0.5.0 | MIT | https://github.com/RazrFalcon/pico-args |
| pin-project | 1.1.13 | Apache-2.0 OR MIT | https://github.com/taiki-e/pin-project |
| pin-project-internal | 1.1.13 | Apache-2.0 OR MIT | https://github.com/taiki-e/pin-project |
| pin-project-lite | 0.2.17 | Apache-2.0 OR MIT | https://github.com/taiki-e/pin-project-lite |
| pin-utils | 0.1.1 | MIT OR Apache-2.0 | https://github.com/rust-lang/pin-utils |
| piper | 0.2.5 | MIT OR Apache-2.0 | https://github.com/smol-rs/piper |
| png | 0.17.16 | MIT OR Apache-2.0 | https://github.com/image-rs/image-png |
| png | 0.18.1 | MIT OR Apache-2.0 | https://github.com/image-rs/image-png |
| polling | 3.11.0 | Apache-2.0 OR MIT | https://github.com/smol-rs/polling |
| pollster | 0.2.5 | Apache-2.0/MIT | https://github.com/zesterer/pollster |
| postage | 0.5.0 | MIT | https://github.com/austinjones/postage-rs |
| potential_utf | 0.1.6 | Unicode-3.0 | https://github.com/unicode-org/icu4x |
| ppv-lite86 | 0.2.21 | MIT OR Apache-2.0 | https://github.com/cryptocorrosion/cryptocorrosion |
| proc-macro-crate | 3.5.0 | MIT OR Apache-2.0 | https://github.com/bkchr/proc-macro-crate |
| proc-macro-error-attr2 | 2.0.0 | MIT OR Apache-2.0 | https://github.com/GnomedDev/proc-macro-error-2 |
| proc-macro-error2 | 2.0.1 | MIT OR Apache-2.0 | https://github.com/GnomedDev/proc-macro-error-2 |
| proc-macro2 | 1.0.107 | MIT OR Apache-2.0 | https://github.com/dtolnay/proc-macro2 |
| profiling | 1.0.18 | MIT OR Apache-2.0 | https://github.com/aclysma/profiling |
| profiling-procmacros | 1.0.18 | MIT OR Apache-2.0 | https://github.com/aclysma/profiling |
| psm | 0.1.32 | MIT OR Apache-2.0 | https://github.com/rust-lang/stacker/ |
| pulp | 0.22.3 | MIT | https://github.com/sarah-quinones/pulp/ |
| pulp-wasm-simd-flag | 0.1.1 | MIT | https://github.com/sarah-quinones/pulp/ |
| pxfm | 0.1.30 | BSD-3-Clause OR Apache-2.0 | https://github.com/awxkee/pxfm |
| qoi | 0.4.1 | MIT/Apache-2.0 | https://github.com/aldanor/qoi-rust |
| quick-error | 2.0.1 | MIT/Apache-2.0 | http://github.com/tailhook/quick-error |
| quick-xml | 0.41.0 | MIT | https://github.com/tafia/quick-xml |
| quote | 1.0.47 | MIT OR Apache-2.0 | https://github.com/dtolnay/quote |
| rand | 0.8.8 | MIT OR Apache-2.0 | https://github.com/rust-random/rand |
| rand | 0.9.5 | MIT OR Apache-2.0 | https://github.com/rust-random/rand |
| rand_chacha | 0.3.1 | MIT OR Apache-2.0 | https://github.com/rust-random/rand |
| rand_chacha | 0.9.0 | MIT OR Apache-2.0 | https://github.com/rust-random/rand |
| rand_core | 0.6.4 | MIT OR Apache-2.0 | https://github.com/rust-random/rand |
| rand_core | 0.9.5 | MIT OR Apache-2.0 | https://github.com/rust-random/rand |
| rangemap | 1.8.0 | MIT/Apache-2.0 | https://github.com/jeffparsons/rangemap |
| rav1e | 0.8.1 | BSD-2-Clause | https://github.com/xiph/rav1e/ |
| ravif | 0.13.0 | BSD-3-Clause | https://github.com/kornelski/cavif-rs |
| raw-cpuid | 11.6.0 | MIT | https://github.com/gz/rust-cpuid |
| raw-window-handle | 0.6.2 | MIT OR Apache-2.0 OR Zlib | https://github.com/rust-windowing/raw-window-handle |
| rayon | 1.12.0 | MIT OR Apache-2.0 | https://github.com/rayon-rs/rayon |
| rayon-core | 1.13.0 | MIT OR Apache-2.0 | https://github.com/rayon-rs/rayon |
| read-fonts | 0.41.0 | MIT OR Apache-2.0 | https://github.com/googlefonts/fontations |
| reborrow | 0.5.5 | MIT | https://github.com/sarah-ek/reborrow/ |
| ref-cast | 1.0.27 | MIT OR Apache-2.0 | https://github.com/dtolnay/ref-cast |
| ref-cast-impl | 1.0.27 | MIT OR Apache-2.0 | https://github.com/dtolnay/ref-cast |
| regex | 1.13.1 | MIT OR Apache-2.0 | https://github.com/rust-lang/regex |
| regex-automata | 0.4.18 | MIT OR Apache-2.0 | https://github.com/rust-lang/regex |
| regex-syntax | 0.8.11 | MIT OR Apache-2.0 | https://github.com/rust-lang/regex |
| resvg | 0.45.1 | Apache-2.0 OR MIT | https://github.com/linebender/resvg |
| rgb | 0.8.53 | MIT | https://github.com/kornelski/rust-rgb |
| ring | 0.17.14 | Apache-2.0 AND ISC | https://github.com/briansmith/ring |
| roxmltree | 0.20.0 | MIT OR Apache-2.0 | https://github.com/RazrFalcon/roxmltree |
| rust-embed | 8.13.0 | MIT | https://pyrossh.dev/repos/rust-embed |
| rust-embed-impl | 8.13.0 | MIT | https://pyrossh.dev/repos/rust-embed |
| rust-embed-utils | 8.13.0 | MIT | https://pyrossh.dev/repos/rust-embed |
| rustc-hash | 1.1.0 | Apache-2.0/MIT | https://github.com/rust-lang-nursery/rustc-hash |
| rustc-hash | 2.1.3 | Apache-2.0 OR MIT | https://github.com/rust-lang/rustc-hash |
| rustix | 0.38.44 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT | https://github.com/bytecodealliance/rustix |
| rustix | 1.1.6 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT | https://github.com/bytecodealliance/rustix |
| rustls | 0.23.46 | Apache-2.0 OR ISC OR MIT | https://github.com/rustls/rustls |
| rustls-native-certs | 0.8.5 | Apache-2.0 OR ISC OR MIT | https://github.com/rustls/rustls-native-certs |
| rustls-pemfile | 2.2.0 | Apache-2.0 OR ISC OR MIT | https://github.com/rustls/pemfile |
| rustls-pki-types | 1.15.1 | MIT OR Apache-2.0 | https://github.com/rustls/pki-types |
| rustls-webpki | 0.103.16 | ISC | https://github.com/rustls/webpki |
| rustversion | 1.0.23 | MIT OR Apache-2.0 | https://github.com/dtolnay/rustversion |
| rustybuzz | 0.14.1 | MIT | https://github.com/RazrFalcon/rustybuzz |
| rustybuzz | 0.20.1 | MIT | https://github.com/harfbuzz/rustybuzz |
| ryu | 1.0.23 | Apache-2.0 OR BSL-1.0 | https://github.com/dtolnay/ryu |
| same-file | 1.0.6 | Unlicense/MIT | https://github.com/BurntSushi/same-file |
| schemars | 1.2.3 | MIT | https://github.com/GREsau/schemars |
| schemars_derive | 1.2.3 | MIT | https://github.com/GREsau/schemars |
| scoped-tls | 1.0.1 | MIT/Apache-2.0 | https://github.com/alexcrichton/scoped-tls |
| scopeguard | 1.2.0 | MIT OR Apache-2.0 | https://github.com/bluss/scopeguard |
| seahash | 4.1.0 | MIT | https://gitlab.redox-os.org/redox-os/seahash |
| self_cell | 1.3.0 | Apache-2.0 OR GPL-2.0-only | https://github.com/Voultapher/self_cell |
| serde | 1.0.229 | MIT OR Apache-2.0 | https://github.com/serde-rs/serde |
| serde_core | 1.0.229 | MIT OR Apache-2.0 | https://github.com/serde-rs/serde |
| serde_derive | 1.0.229 | MIT OR Apache-2.0 | https://github.com/serde-rs/serde |
| serde_derive_internals | 0.30.0 | MIT OR Apache-2.0 | https://github.com/serde-rs/serde |
| serde_fmt | 1.1.0 | Apache-2.0 OR MIT | https://github.com/KodrAus/serde_fmt.git |
| serde_json | 1.0.154 | MIT OR Apache-2.0 | https://github.com/serde-rs/json |
| serde_json_lenient | 0.2.4 | MIT/Apache-2.0 | https://github.com/google/serde_json_lenient |
| serde_repr | 0.1.21 | MIT OR Apache-2.0 | https://github.com/dtolnay/serde-repr |
| serde_urlencoded | 0.7.1 | MIT/Apache-2.0 | https://github.com/nox/serde_urlencoded |
| sha1_smol | 1.0.1 | BSD-3-Clause | https://github.com/mitsuhiko/sha1-smol |
| sha2 | 0.10.9 | MIT OR Apache-2.0 | https://github.com/RustCrypto/hashes |
| sha2 | 0.11.1 | MIT OR Apache-2.0 | https://github.com/RustCrypto/hashes |
| shlex | 1.3.0 | MIT OR Apache-2.0 | https://github.com/comex/rust-shlex |
| signal-hook-registry | 1.4.8 | MIT OR Apache-2.0 | https://github.com/vorner/signal-hook |
| simd-adler32 | 0.3.10 | MIT | https://github.com/mcountryman/simd-adler32 |
| simd_helpers | 0.1.0 | MIT | https://github.com/lu-zero/simd_helpers |
| simdutf8 | 0.1.5 | MIT OR Apache-2.0 | https://github.com/rusticstuff/simdutf8 |
| simplecss | 0.2.2 | Apache-2.0 OR MIT | https://github.com/linebender/simplecss |
| siphasher | 1.0.4 | MIT OR Apache-2.0 | https://github.com/jedisct1/rust-siphash |
| skrifa | 0.44.0 | MIT OR Apache-2.0 | https://github.com/googlefonts/fontations |
| slab | 0.4.12 | MIT | https://github.com/tokio-rs/slab |
| slotmap | 1.1.1 | Zlib | https://github.com/orlp/slotmap |
| smallvec | 1.16.5 | MIT OR Apache-2.0 | https://github.com/servo/rust-smallvec |
| smol | 2.0.2 | Apache-2.0 OR MIT | https://github.com/smol-rs/smol |
| smol_str | 0.2.2 | MIT OR Apache-2.0 | https://github.com/rust-analyzer/smol_str |
| socket2 | 0.6.5 | MIT OR Apache-2.0 | https://github.com/rust-lang/socket2 |
| spin | 0.9.9 | MIT | https://github.com/mvdnes/spin-rs.git |
| spirv | 0.3.0+sdk-1.3.268.0 | Apache-2.0 | https://github.com/gfx-rs/rspirv |
| stable_deref_trait | 1.2.1 | MIT OR Apache-2.0 | https://github.com/storyyeller/stable_deref_trait |
| stacker | 0.1.25 | MIT OR Apache-2.0 | https://github.com/rust-lang/stacker |
| stacksafe | 0.1.4 | Apache-2.0 | https://github.com/fast/stacksafe |
| stacksafe-macro | 0.1.4 | Apache-2.0 | https://github.com/fast/stacksafe |
| static_assertions | 1.1.0 | MIT OR Apache-2.0 | https://github.com/nvzqz/static-assertions-rs |
| strict-num | 0.1.1 | MIT | https://github.com/RazrFalcon/strict-num |
| strum | 0.26.3 | MIT | https://github.com/Peternator7/strum |
| strum | 0.27.2 | MIT | https://github.com/Peternator7/strum |
| strum_macros | 0.26.4 | MIT | https://github.com/Peternator7/strum |
| strum_macros | 0.27.2 | MIT | https://github.com/Peternator7/strum |
| subtle | 2.6.1 | BSD-3-Clause | https://github.com/dalek-cryptography/subtle |
| svg_fmt | 0.4.5 | MIT/Apache-2.0 | https://github.com/nical/rust_debug |
| svgtypes | 0.15.3 | Apache-2.0 OR MIT | https://github.com/linebender/svgtypes |
| swash | 0.2.10 | Apache-2.0 OR MIT | https://github.com/dfrg/swash |
| syn | 1.0.109 | MIT OR Apache-2.0 | https://github.com/dtolnay/syn |
| syn | 2.0.119 | MIT OR Apache-2.0 | https://github.com/dtolnay/syn |
| syn | 3.0.9 | MIT OR Apache-2.0 | https://github.com/dtolnay/syn |
| sync_wrapper | 1.0.2 | Apache-2.0 | https://github.com/Actyx/sync_wrapper |
| synstructure | 0.14.0 | MIT | https://github.com/mystor/synstructure |
| sys-locale | 0.3.2 | MIT OR Apache-2.0 | https://github.com/1Password/sys-locale |
| taffy | 0.9.0 | MIT | https://github.com/DioxusLabs/taffy |
| take-until | 0.2.0 | MIT | https://github.com/hdevalke/take-until.git |
| tempfile | 3.27.0 | MIT OR Apache-2.0 | https://github.com/Stebalien/tempfile |
| termcolor | 1.4.1 | Unlicense OR MIT | https://github.com/BurntSushi/termcolor |
| thiserror | 1.0.69 | MIT OR Apache-2.0 | https://github.com/dtolnay/thiserror |
| thiserror | 2.0.21 | MIT OR Apache-2.0 | https://github.com/dtolnay/thiserror |
| thiserror-impl | 1.0.69 | MIT OR Apache-2.0 | https://github.com/dtolnay/thiserror |
| thiserror-impl | 2.0.21 | MIT OR Apache-2.0 | https://github.com/dtolnay/thiserror |
| tiff | 0.11.3 | MIT | https://github.com/image-rs/image-tiff |
| tiny-keccak | 2.0.2 | CC0-1.0 | https://github.com/debris/tiny-keccak |
| tiny-skia | 0.11.4 | BSD-3-Clause | https://github.com/RazrFalcon/tiny-skia |
| tiny-skia-path | 0.11.4 | BSD-3-Clause | https://github.com/RazrFalcon/tiny-skia/tree/master/path |
| tinystr | 0.8.5 | Unicode-3.0 | https://github.com/unicode-org/icu4x |
| tinyvec | 1.13.3 | Zlib OR Apache-2.0 OR MIT | https://github.com/Lokathor/tinyvec |
| tokio | 1.54.1 | MIT | https://github.com/tokio-rs/tokio |
| tokio-rustls | 0.26.6 | MIT OR Apache-2.0 | https://github.com/rustls/tokio-rustls |
| tokio-socks | 0.5.3 | MIT | https://github.com/sticnarf/tokio-socks |
| tokio-util | 0.7.20 | MIT | https://github.com/tokio-rs/tokio |
| toml_datetime | 1.1.3+spec-1.1.0 | MIT OR Apache-2.0 | https://github.com/toml-rs/toml |
| toml_edit | 0.25.19+spec-1.1.0 | MIT OR Apache-2.0 | https://github.com/toml-rs/toml |
| toml_parser | 1.1.5+spec-1.1.0 | MIT OR Apache-2.0 | https://github.com/toml-rs/toml |
| tower | 0.5.3 | MIT | https://github.com/tower-rs/tower |
| tower-layer | 0.3.3 | MIT | https://github.com/tower-rs/tower |
| tower-service | 0.3.3 | MIT | https://github.com/tower-rs/tower |
| tracing | 0.1.44 | MIT | https://github.com/tokio-rs/tracing |
| tracing-attributes | 0.1.31 | MIT | https://github.com/tokio-rs/tracing |
| tracing-core | 0.1.36 | MIT | https://github.com/tokio-rs/tracing |
| try-lock | 0.2.5 | MIT | https://github.com/seanmonstar/try-lock |
| ttf-parser | 0.20.0 | MIT OR Apache-2.0 | https://github.com/RazrFalcon/ttf-parser |
| ttf-parser | 0.21.1 | MIT OR Apache-2.0 | https://github.com/RazrFalcon/ttf-parser |
| ttf-parser | 0.25.1 | MIT OR Apache-2.0 | https://github.com/harfbuzz/ttf-parser |
| typeid | 1.0.3 | MIT OR Apache-2.0 | https://github.com/dtolnay/typeid |
| typenum | 1.20.1 | MIT OR Apache-2.0 | https://github.com/paholg/typenum |
| unicase | 2.10.0 | MIT OR Apache-2.0 | https://github.com/seanmonstar/unicase |
| unicode-bidi | 0.3.18 | MIT OR Apache-2.0 | https://github.com/servo/unicode-bidi |
| unicode-bidi-mirroring | 0.2.0 | MIT/Apache-2.0 | https://github.com/RazrFalcon/unicode-bidi-mirroring |
| unicode-bidi-mirroring | 0.4.0 | MIT/Apache-2.0 | https://github.com/RazrFalcon/unicode-bidi-mirroring |
| unicode-ccc | 0.2.0 | MIT/Apache-2.0 | https://github.com/RazrFalcon/unicode-ccc |
| unicode-ccc | 0.4.0 | MIT/Apache-2.0 | https://github.com/RazrFalcon/unicode-ccc |
| unicode-ident | 1.0.27 | (MIT OR Apache-2.0) AND Unicode-3.0 | https://github.com/dtolnay/unicode-ident |
| unicode-linebreak | 0.1.5 | Apache-2.0 | https://github.com/axelf4/unicode-linebreak |
| unicode-properties | 0.1.5 | (MIT OR Apache-2.0) AND Unicode-3.0 | https://github.com/unicode-rs/unicode-properties |
| unicode-script | 0.5.9 | MIT OR Apache-2.0 | https://github.com/unicode-rs/unicode-script |
| unicode-segmentation | 1.13.3 | MIT OR Apache-2.0 | https://github.com/unicode-rs/unicode-segmentation |
| unicode-vo | 0.1.0 | MIT/Apache-2.0 | https://github.com/RazrFalcon/unicode-vo |
| unicode-width | 0.2.3 | MIT OR Apache-2.0 | https://github.com/unicode-rs/unicode-width |
| untrusted | 0.9.0 | ISC | https://github.com/briansmith/untrusted |
| url | 2.5.8 | MIT OR Apache-2.0 | https://github.com/servo/rust-url |
| usvg | 0.45.1 | Apache-2.0 OR MIT | https://github.com/linebender/resvg |
| utf8_iter | 1.1.0 | Apache-2.0 OR MIT | https://github.com/hsivonen/utf8_iter |
| utf_types | 1.0.0 | Apache-2.0 OR MIT | https://github.com/hsivonen/utf_types |
| uuid | 1.29.1 | Apache-2.0 OR MIT | https://github.com/uuid-rs/uuid |
| v_frame | 0.3.9 | BSD-2-Clause | https://github.com/rust-av/v_frame |
| value-bag | 1.14.1 | Apache-2.0 OR MIT | https://github.com/sval-rs/value-bag |
| value-bag-serde1 | 1.14.1 | Apache-2.0 OR MIT |  |
| waker-fn | 1.2.0 | Apache-2.0 OR MIT | https://github.com/smol-rs/waker-fn |
| walkdir | 2.5.0 | Unlicense/MIT | https://github.com/BurntSushi/walkdir |
| want | 0.3.2 | MIT | https://github.com/seanmonstar/want |
| wayland-backend | 0.3.17 | MIT | https://github.com/smithay/wayland-rs |
| wayland-client | 0.31.15 | MIT | https://github.com/smithay/wayland-rs |
| wayland-cursor | 0.31.14 | MIT | https://github.com/smithay/wayland-rs |
| wayland-protocols | 0.31.2 | MIT | https://github.com/smithay/wayland-rs |
| wayland-protocols | 0.32.13 | MIT | https://github.com/smithay/wayland-rs |
| wayland-protocols-plasma | 0.2.0 | MIT | https://github.com/smithay/wayland-rs |
| wayland-scanner | 0.31.11 | MIT | https://github.com/smithay/wayland-rs |
| wayland-sys | 0.31.11 | MIT | https://github.com/smithay/wayland-rs |
| weezl | 0.1.12 | MIT OR Apache-2.0 | https://github.com/image-rs/weezl |
| which | 6.0.3 | MIT | https://github.com/harryfei/which-rs.git |
| winnow | 1.0.5 | MIT | https://github.com/winnow-rs/winnow |
| writeable | 0.6.4 | Unicode-3.0 | https://github.com/unicode-org/icu4x |
| x11-clipboard | 0.9.3 | MIT | https://github.com/quininer/x11-clipboard |
| x11rb | 0.13.2 | MIT OR Apache-2.0 | https://github.com/psychon/x11rb |
| x11rb-protocol | 0.13.2 | MIT OR Apache-2.0 | https://github.com/psychon/x11rb |
| xattr | 0.2.3 | MIT/Apache-2.0 | https://github.com/Stebalien/xattr |
| xcursor | 0.3.11 | MIT | https://github.com/esposm03/xcursor-rs |
| xim-ctext | 0.3.1 | MIT | https://github.com/Riey/xim-rs |
| xim-parser | 0.2.2 | MIT | https://github.com/Riey/xim-rs |
| xkbcommon | 0.8.0 | MIT | https://github.com/rust-x-bindings/xkbcommon-rs |
| xkeysym | 0.2.1 | MIT OR Apache-2.0 OR Zlib | https://github.com/notgull/xkeysym |
| xmlwriter | 0.1.0 | MIT | https://github.com/RazrFalcon/xmlwriter |
| y4m | 0.8.0 | MIT | https://github.com/image-rs/y4m.git |
| yazi | 0.2.2 | Apache-2.0 OR MIT | https://github.com/dfrg/yazi |
| yeslogic-fontconfig-sys | 6.0.1 | MIT | https://github.com/yeslogic/fontconfig-rs |
| yoke | 0.8.4 | Unicode-3.0 | https://github.com/unicode-org/icu4x |
| yoke-derive | 0.8.4 | Unicode-3.0 | https://github.com/unicode-org/icu4x |
| zbus | 5.19.0 | MIT | https://github.com/z-galaxy/zbus/ |
| zbus_macros | 5.19.0 | MIT | https://github.com/z-galaxy/zbus/ |
| zbus_names | 4.3.5 | MIT | https://github.com/z-galaxy/zbus/ |
| zcheapstr | 1.2.0 | MIT | https://github.com/z-galaxy/zcheapstr/ |
| zed-async-tar | 0.5.0-zed | MIT/Apache-2.0 | https://github.com/dignifiedquire/async-tar |
| zed-font-kit | 0.14.1-zed | MIT OR Apache-2.0 | https://github.com/servo/font-kit |
| zed-reqwest | 0.12.15-zed | MIT OR Apache-2.0 | https://github.com/seanmonstar/reqwest |
| zed-xim | 0.4.0-zed | MIT | https://github.com/Riey/xim-rs |
| zeno | 0.3.3 | Apache-2.0 OR MIT | https://github.com/dfrg/zeno |
| zerocopy | 0.8.63 | BSD-2-Clause OR Apache-2.0 OR MIT | https://github.com/google/zerocopy |
| zerocopy-derive | 0.8.63 | BSD-2-Clause OR Apache-2.0 OR MIT | https://github.com/google/zerocopy |
| zerofrom | 0.1.8 | Unicode-3.0 | https://github.com/unicode-org/icu4x |
| zerofrom-derive | 0.1.8 | Unicode-3.0 | https://github.com/unicode-org/icu4x |
| zeroize | 1.9.1 | Apache-2.0 OR MIT | https://github.com/RustCrypto/utils |
| zeroize_derive | 1.5.0 | Apache-2.0 OR MIT | https://github.com/RustCrypto/utils |
| zerotrie | 0.2.5 | Unicode-3.0 | https://github.com/unicode-org/icu4x |
| zerovec | 0.11.9 | Unicode-3.0 | https://github.com/unicode-org/icu4x |
| zerovec-derive | 0.11.7 | Unicode-3.0 | https://github.com/unicode-org/icu4x |
| zmij | 1.0.23 | MIT | https://github.com/dtolnay/zmij |
| zune-core | 0.5.3 | MIT OR Apache-2.0 OR Zlib | https://github.com/etemesi254/zune-image |
| zune-inflate | 0.2.54 | MIT OR Apache-2.0 OR Zlib | https://github.com/etemesi254/zune-image/tree/main/zune-inflate |
| zune-jpeg | 0.5.15 | MIT OR Apache-2.0 OR Zlib | https://github.com/etemesi254/zune-image/tree/dev/crates/zune-jpeg |
| zvariant | 5.15.1 | MIT | https://github.com/z-galaxy/zbus/ |
| zvariant_derive | 5.15.1 | MIT | https://github.com/z-galaxy/zbus/ |
| zvariant_utils | 4.2.0 | MIT | https://github.com/z-galaxy/zbus/ |
