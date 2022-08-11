use criterion::{black_box, criterion_group, criterion_main, Criterion};

macro_rules! auto_bench {
    ($([$name: ident, $path: expr] $(,)? )*,) => {
        fn criterion_benchmark(c: &mut Criterion) {
            $(
                c.bench_function(concat!("parse_", $path), |b| {
                    let input_bytes = include_bytes!(concat!("../tests/swfs/", $path, ".swf"));
                    let input_bytes = &input_bytes[..];
                    b.iter(|| {
                        let stream = black_box(swf::decompress_swf(input_bytes).unwrap());
                        let swf = black_box(swf::parse_swf(&stream).unwrap());
                        return black_box((swf.header, swf.tags.len()));
                    })
                });
            )*
        }
    }
}

auto_bench! {
    [bench_1, "Avm2Dummy"],
    [bench_2, "BitmapLineStyle"],
    [bench_3, "DefineBinaryData"],
    [bench_4, "DefineBits-JpegTables-MX"],
    [bench_5, "DefineBitsJpeg2-MX"],
    [bench_6, "DefineBitsJpeg3"],
    [bench_7, "DefineBitsLossless"],
    [bench_8, "DefineBitsLossless2"],
    [bench_9, "DefineButton-MX"],
    [bench_10, "DefineButton2-CS6"],
    [bench_11, "DefineButtonCxformSound-MX"],
    [bench_12, "DefineEditText-MX"],
    [bench_13, "DefineFont-MX"],
    [bench_14, "DefineFont2-CS6"],
    [bench_15, "DefineFont3-CS6"],
    [bench_16, "DefineFont3-DeviceText"],
    [bench_17, "DefineFont4"],
    [bench_18, "DefineMorphShape-MX"],
    [bench_19, "DefineMorphShape2-GradientFlags"],
    [bench_20, "DefineMorphShape2"],
    [bench_21, "DefineScalingGrid"],
    [bench_22, "DefineSceneAndFrameLabelData"],
    [bench_23, "DefineShape"],
    [bench_24, "DefineShape3"],
    [bench_25, "DefineShape4"],
    [bench_26, "DefineSound"],
    [bench_27, "DefineSprite"],
    [bench_28, "DefineText2-MX"],
    [bench_29, "DefineVideoStream"],
    [bench_30, "DoAction-CS6"],
    [bench_31, "DoInitAction-CS6"],
    [bench_32, "EnableDebugger2-CS6"],
    [bench_33, "EnableTelemetry-password"],
    [bench_34, "EnableTelemetry"],
    [bench_35, "ExportAssets-CS6"],
    [bench_36, "FrameLabel-CS6"],
    [bench_37, "ImportAssets-CS6"],
    [bench_38, "ImportAssets2-CS6"],
    [bench_39, "PlaceObject2-ClipActions-CS6"],
    [bench_40, "PlaceObject2-ClipActionsV5-CS6"],
    [bench_41, "PlaceObject3-Image"],
    [bench_42, "PlaceObject3-theworks"],
    [bench_43, "PlaceObject4"],
    [bench_44, "Protect"],
    [bench_45, "ProtectNoPassword"],
    [bench_46, "ScriptLimits"],
    [bench_47, "SimpleRedBackground"],
    [bench_48, "SoundStreamHead2"],
    [bench_49, "StartSound2"],
    [bench_50, "SymbolClass"],
    [bench_51, "lzma"],
    [bench_52, "uncompressed"],
    [bench_53, "zlib"],
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);