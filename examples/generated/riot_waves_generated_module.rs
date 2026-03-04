pub mod riot_waves_generated {

    use arthropod::figma::{FigmaImportError, ImportedFigmaDocument, import_figma_document};
    use arthropod::figma_runtime::{FigmaRuntime, FigmaRuntimeError};

    pub const SOURCE_BYTES: usize = 50038;
    pub const SOURCE_FNV64: u64 = 0xa794d13089c5185c;
    pub const NODE_COUNT: usize = 74;
    pub const PROTOTYPE_EDGE_COUNT: usize = 0;

    pub const FIGMA_JSON: &str = r#"{
  "nodes": [
    {
      "bounds": [
        0.0,
        0.0,
        1366.0,
        884.0
      ],
      "clipsContent": true,
      "fills": [
        {
          "color": [
            0.019607843831181526,
            0.019607843831181526,
            0.019607843831181526,
            1.0
          ],
          "type": "SOLID",
          "visible": true
        }
      ],
      "id": "stitch:0",
      "layoutMode": "VERTICAL",
      "type": "FRAME"
    },
    {
      "bounds": [
        0.0,
        0.0,
        1366.0,
        884.0
      ],
      "fills": [
        {
          "color": [
            0.10980392247438431,
            0.10980392247438431,
            0.10980392247438431,
            1.0
          ],
          "type": "SOLID",
          "visible": true
        }
      ],
      "id": "stitch:0.1",
      "layoutGrow": 1.0,
      "parentId": "stitch:0",
      "type": "FRAME"
    },
    {
      "bounds": [
        0.0,
        0.0,
        1366.0,
        751.4000244140625
      ],
      "clipsContent": true,
      "fills": [
        {
          "color": [
            0.0,
            0.0,
            0.0,
            1.0
          ],
          "type": "SOLID",
          "visible": true
        }
      ],
      "id": "stitch:0.1.0",
      "parentId": "stitch:0.1",
      "type": "FRAME"
    },
    {
      "bounds": [
        0.0,
        0.0,
        1366.0,
        751.4000244140625
      ],
      "fills": [
        {
          "color": [
            0.06666667014360428,
            0.0941176488995552,
            0.15294118225574493,
            1.0
          ],
          "type": "SOLID",
          "visible": true
        }
      ],
      "id": "stitch:0.1.0.0",
      "layoutPositioning": "ABSOLUTE",
      "parentId": "stitch:0.1.0",
      "type": "FRAME"
    },
    {
      "bounds": [
        0.0,
        0.0,
        1366.0,
        751.4000244140625
      ],
      "clipsContent": true,
      "fills": [
        {
          "imageFilter": {
            "contrast": 1.5,
            "grayscale": 1.0,
            "invert": 1.0
          },
          "imageRef": "filtered://composite://halftone/assets/lh3_googleusercontent_com_fbf53e3f8caee692.png#5ce2c97a3325c8d4",
          "imageSourceRef": "composite://halftone/assets/lh3_googleusercontent_com_fbf53e3f8caee692.png",
          "scaleMode": "FILL",
          "type": "IMAGE",
          "visible": true
        }
      ],
      "id": "stitch:0.1.0.0.0",
      "opacity": 0.800000011920929,
      "parentId": "stitch:0.1.0.0",
      "type": "RECTANGLE"
    },
    {
      "blendMode": "OVERLAY",
      "bounds": [
        0.0,
        0.0,
        1366.0,
        751.4000244140625
      ],
      "fills": [
        {
          "imageRef": "procedural://noise/294f247ca9ff6ac5",
          "scaleMode": "TILE",
          "type": "IMAGE",
          "visible": true
        }
      ],
      "id": "stitch:0.1.0.0.1",
      "layoutPositioning": "ABSOLUTE",
      "opacity": 0.4000000059604645,
      "parentId": "stitch:0.1.0.0",
      "type": "RECTANGLE"
    },
    {
      "bounds": [
        0.0,
        0.0,
        1366.0,
        751.4000244140625
      ],
      "id": "stitch:0.1.0.1",
      "layoutMode": "VERTICAL",
      "layoutPositioning": "ABSOLUTE",
      "paddingBottom": 96.0,
      "paddingLeft": 24.0,
      "paddingRight": 24.0,
      "paddingTop": 24.0,
      "parentId": "stitch:0.1.0",
      "primaryAxisAlignItems": "MAX",
      "type": "FRAME"
    },
    {
      "bounds": [
        1246.0,
        104.0,
        80.0,
        80.0
      ],
      "id": "stitch:0.1.0.1.0",
      "layoutPositioning": "ABSOLUTE",
      "opacity": 0.8999999761581421,
      "parentId": "stitch:0.1.0.1",
      "rotation": 12.0,
      "type": "FRAME"
    },
    {
      "blendMode": "SCREEN",
      "bounds": [
        1246.0,
        104.0,
        80.0,
        80.0
      ],
      "cornerRadius": 40.0,
      "counterAxisAlignItems": "CENTER",
      "id": "stitch:0.1.0.1.0.0",
      "layoutMode": "HORIZONTAL",
      "parentId": "stitch:0.1.0.1.0",
      "primaryAxisAlignItems": "CENTER",
      "strokeAlign": "CENTER",
      "strokeWeight": 4.0,
      "strokes": [
        {
          "color": [
            0.800000011920929,
            1.0,
            0.0,
            1.0
          ],
          "type": "SOLID",
          "visible": true
        }
      ],
      "type": "FRAME"
    },
    {
      "blendMode": "SCREEN",
      "bounds": [
        1265.0,
        126.5,
        42.0,
        35.0
      ],
      "characters": "TODAY\nONLY",
      "effects": [
        {
          "color": [
            0.800000011920929,
            1.0,
            0.0,
            0.800000011920929
          ],
          "offset": [
            0.0,
            0.0
          ],
          "radius": 5.0,
          "type": "DROP_SHADOW",
          "visible": true
        }
      ],
      "fills": [
        {
          "color": [
            0.800000011920929,
            1.0,
            0.0,
            1.0
          ],
          "type": "SOLID",
          "visible": true
        },
        {
          "color": [
            0.0,
            0.0,
            0.0,
            0.0
          ],
          "type": "SOLID",
          "visible": true
        }
      ],
      "id": "stitch:0.1.0.1.0.0.0",
      "parentId": "stitch:0.1.0.1.0.0",
      "style": {
        "fontFamily": "Special Elite",
        "fontSize": 14.0,
        "fontWeight": 700,
        "lineHeightPercentFontSize": 125.0,
        "textAlignHorizontal": "CENTER"
      },
      "type": "TEXT"
    },
    {
      "bounds": [
        24.0,
        431.10003662109375,
        1318.0,
        144.0
      ],
      "id": "stitch:0.1.0.1.1",
      "parentId": "stitch:0.1.0.1",
      "type": "FRAME"
    },
    {
      "bounds": [
        24.0,
        431.10003662109375,
        1318.0,
        144.0
      ],
      "id": "stitch:0.1.0.1.1.0",
      "parentId": "stitch:0.1.0.1.1",
      "rotation": -3.0,
      "type": "FRAME"
    },
    {
      "bounds": [
        24.0,
        431.10003662109375,
        188.8000030517578,
        72.0
      ],
      "characters": "SCUM",
      "effects": [
        {
          "color": [
            0.800000011920929,
            1.0,
            0.0,
            1.0
          ],
          "offset": [
            4.0,
            4.0
          ],
          "radius": 0.0,
          "type": "DROP_SHADOW",
          "visible": true
        }
      ],
      "fills": [
        {
          "color": [
            0.0,
            0.0,
            0.0,
            1.0
          ],
          "type": "SOLID",
          "visible": true
        },
        {
          "color": [
            1.0,
            1.0,
            1.0,
            1.0
          ],
          "type": "SOLID",
          "visible": true
        }
      ],
      "id": "stitch:0.1.0.1.1.0.0",
      "paddingLeft": 8.0,
      "paddingRight": 8.0,
      "parentId": "stitch:0.1.0.1.1.0",
      "rotation": -1.0,
      "style": {
        "fontFamily": "Sedgwick Ave Display",
        "fontSize": 72.0,
        "lineHeightPercentFontSize": 100.0
      },
      "type": "TEXT"
    },
    {
      "bounds": [
        56.0,
        503.10003662109375,
        232.00001525878906,
        72.0
      ],
      "characters": "FUCKS",
      "effects": [
        {
          "color": [
            0.800000011920929,
            1.0,
            0.0,
            1.0
          ],
          "offset": [
            4.0,
            4.0
          ],
          "radius": 0.0,
          "type": "DROP_SHADOW",
          "visible": true
        }
      ],
      "fills": [
        {
          "color": [
            0.0,
            0.0,
            0.0,
            1.0
          ],
          "type": "SOLID",
          "visible": true
        },
        {
          "color": [
            1.0,
            1.0,
            1.0,
            1.0
          ],
          "type": "SOLID",
          "visible": true
        }
      ],
      "id": "stitch:0.1.0.1.1.0.1",
      "paddingLeft": 8.0,
      "paddingRight": 8.0,
      "parentId": "stitch:0.1.0.1.1.0",
      "rotation": 2.0,
      "style": {
        "fontFamily": "Sedgwick Ave Display",
        "fontSize": 72.0,
        "lineHeightPercentFontSize": 100.0
      },
      "type": "TEXT"
    },
    {
      "bounds": [
        24.0,
        599.1000366210938,
        209.40000915527344,
        40.30000305175781
      ],
      "id": "stitch:0.1.0.1.2",
      "layoutAlign": "MIN",
      "parentId": "stitch:0.1.0.1",
      "rotation": -2.0,
      "type": "FRAME"
    },
    {
      "bounds": [
        24.0,
        599.1000366210938,
        209.40000915527344,
        40.30000305175781
      ],
      "counterAxisAlignItems": "CENTER",
      "effects": [
        {
          "color": [
            0.0,
            0.0,
            0.0,
            0.800000011920929
          ],
          "offset": [
            0.0,
            1.0
          ],
          "radius": 3.0,
          "type": "DROP_SHADOW",
          "visible": true
        }
      ],
      "fills": [
        {
          "color": [
            0.11764705926179886,
            0.11764705926179886,
            0.11764705926179886,
            0.949999988079071
          ],
          "type": "SOLID",
          "visible": true
        }
      ],
      "id": "stitch:0.1.0.1.2.0",
      "itemSpacing": 8.0,
      "layoutMode": "HORIZONTAL",
      "paddingBottom": 8.0,
      "paddingLeft": 16.0,
      "paddingRight": 16.0,
      "paddingTop": 8.0,
      "parentId": "stitch:0.1.0.1.2",
      "rotation": -1.0,
      "strokeAlign": "CENTER",
      "strokeWeight": 1.0,
      "strokes": [
        {
          "color": [
            1.0,
            1.0,
            1.0,
            0.10000000149011612
          ],
          "type": "SOLID",
          "visible": true
        }
      ],
      "type": "FRAME"
    },
    {
      "bounds": [
        40.0,
        609.2500610351562,
        20.0,
        20.0
      ],
      "characters": "graphic_eq",
      "fills": [
        {
          "color": [
            0.800000011920929,
            1.0,
            0.0,
            1.0
          ],
          "type": "SOLID",
          "visible": true
        },
        {
          "color": [
            0.0,
            0.0,
            0.0,
            0.0
          ],
          "type": "SOLID",
          "visible": true
        }
      ],
      "id": "stitch:0.1.0.1.2.0.0",
      "parentId": "stitch:0.1.0.1.2.0",
      "style": {
        "fontFamily": "Material Symbols Outlined",
        "fontSize": 20.0,
        "fontStyle": "NORMAL",
        "fontWeight": 400,
        "letterSpacing": 0.0,
        "lineHeightPercentFontSize": 100.0
      },
      "type": "TEXT"
    },
    {
      "bounds": [
        68.0,
        607.1000366210938,
        149.40000915527344,
        24.30000114440918
      ],
      "characters": "\"RAT POISON\"",
      "fills": [
        {
          "color": [
            0.8980392217636108,
            0.9058823585510254,
            0.9215686321258545,
            1.0
          ],
          "type": "SOLID",
          "visible": true
        },
        {
          "color": [
            0.0,
            0.0,
            0.0,
            0.0
          ],
          "type": "SOLID",
          "visible": true
        }
      ],
      "id": "stitch:0.1.0.1.2.0.1",
      "parentId": "stitch:0.1.0.1.2.0",
      "style": {
        "fontFamily": "Special Elite",
        "fontSize": 18.0,
        "fontWeight": 700,
        "letterSpacing": 1.8000000715255737,
        "textCase": "UPPER"
      },
      "type": "TEXT"
    },
    {
      "bounds": [
        1178.0,
        516.4000244140625,
        140.0,
        43.0
      ],
      "id": "stitch:0.1.0.1.3",
      "layoutPositioning": "ABSOLUTE",
      "parentId": "stitch:0.1.0.1",
      "type": "FRAME"
    },
    {
      "bounds": [
        1178.0,
        516.4000244140625,
        140.0,
        43.0
      ],
      "characters": "PLAY LOUD",
      "effects": [
        {
          "color": [
            1.0,
            1.0,
            1.0,
            1.0
          ],
          "offset": [
            4.0,
            4.0
          ],
          "radius": 0.0,
          "type": "DROP_SHADOW",
          "visible": true
        }
      ],
      "fills": [
        {
          "color": [
            0.0,
            0.0,
            0.0,
            1.0
          ],
          "type": "SOLID",
          "visible": true
        },
        {
          "color": [
            0.800000011920929,
            1.0,
            0.0,
            1.0
          ],
          "type": "SOLID",
          "visible": true
        }
      ],
      "id": "stitch:0.1.0.1.3.0",
      "paddingBottom": 8.0,
      "paddingLeft": 16.0,
      "paddingRight": 16.0,
      "paddingTop": 8.0,
      "parentId": "stitch:0.1.0.1.3",
      "rotation": 3.0,
      "strokeAlign": "CENTER",
      "strokeWeight": 2.0,
      "strokes": [
        {
          "color": [
            1.0,
            1.0,
            1.0,
            1.0
          ],
          "type": "SOLID",
          "visible": true
        }
      ],
      "style": {
        "fontFamily": "Fugaz One",
        "fontSize": 20.0
      },
      "type": "TEXT"
    },
    {
      "bounds": [
        0.0,
        751.4000244140625,
        1366.0,
        751.4000244140625
      ],
      "clipsContent": true,
      "fills": [
        {
          "color": [
            0.10980392247438431,
            0.10980392247438431,
            0.10980392247438431,
            1.0
          ],
          "type": "SOLID",
          "visible": true
        }
      ],
      "id": "stitch:0.1.1",
      "parentId": "stitch:0.1",
      "type": "FRAME"
    },
    {
      "bounds": [
        0.0,
        751.4000244140625,
        1366.0,
        751.4000244140625
      ],
      "fills": [
        {
          "color": [
            0.0,
            0.0,
            0.0,
            1.0
          ],
          "type": "SOLID",
          "visible": true
        }
      ],
      "id": "stitch:0.1.1.0",
      "layoutPositioning": "ABSOLUTE",
      "parentId": "stitch:0.1.1",
      "type": "FRAME"
    },
    {
      "bounds": [
        0.0,
        751.4000244140625,
        1366.0,
        751.4000244140625
      ],
      "clipsContent": true,
      "fills": [
        {
          "imageFilter": {
            "contrast": 1.5,
            "grayscale": 1.0,
            "invert": 1.0
          },
          "imageRef": "filtered://composite://halftone/assets/lh3_googleusercontent_com_9d79993df332b436.png#f5a47268e99986fa",
          "imageSourceRef": "composite://halftone/assets/lh3_googleusercontent_com_9d79993df332b436.png",
          "scaleMode": "FILL",
          "type": "IMAGE",
          "visible": true
        }
      ],
      "id": "stitch:0.1.1.0.0",
      "opacity": 0.699999988079071,
      "parentId": "stitch:0.1.1.0",
      "type": "RECTANGLE"
    },
    {
      "blendMode": "SCREEN",
      "bounds": [
        0.0,
        751.4000244140625,
        1366.0,
        751.4000244140625
      ],
      "fills": [
        {
          "imageRef": "procedural://noise/294f247ca9ff6ac5",
          "scaleMode": "TILE",
          "type": "IMAGE",
          "visible": true
        }
      ],
      "id": "stitch:0.1.1.0.1",
      "layoutPositioning": "ABSOLUTE",
      "opacity": 0.30000001192092896,
      "parentId": "stitch:0.1.1.0",
      "type": "RECTANGLE"
    },
    {
      "bounds": [
        0.0,
        751.4000244140625,
        1366.0,
        751.4000244140625
      ],
      "id": "stitch:0.1.1.1",
      "layoutMode": "VERTICAL",
      "layoutPositioning": "ABSOLUTE",
      "paddingBottom": 96.0,
      "paddingLeft": 24.0,
      "paddingRight": 24.0,
      "paddingTop": 24.0,
      "parentId": "stitch:0.1.1",
      "primaryAxisAlignItems": "MAX",
      "type": "FRAME"
    },
    {
      "bounds": [
        40.0,
        871.4000244140625,
        96.0,
        96.0
      ],
      "id": "stitch:0.1.1.1.0",
      "layoutPositioning": "ABSOLUTE",
      "opacity": 0.8999999761581421,
      "parentId": "stitch:0.1.1.1",
      "rotation": -6.0,
      "type": "FRAME"
    },
    {
      "blendMode": "LINEAR_DODGE",
      "bounds": [
        40.0,
        871.4000244140625,
        96.0,
        96.0
      ],
      "cornerRadius": 48.0,
      "counterAxisAlignItems": "CENTER",
      "effects": [
        {
          "color": [
            1.0,
            0.0,
            1.0,
            0.4000000059604645
          ],
          "offset": [
            0.0,
            0.0
          ],
          "radius": 15.0,
          "type": "DROP_SHADOW",
          "visible": true
        }
      ],
      "id": "stitch:0.1.1.1.0.0",
      "layoutMode": "HORIZONTAL",
      "parentId": "stitch:0.1.1.1.0",
      "primaryAxisAlignItems": "CENTER",
      "strokeAlign": "CENTER",
      "strokeWeight": 3.0,
      "strokes": [
        {
          "color": [
            1.0,
            0.0,
            1.0,
            1.0
          ],
          "type": "SOLID",
          "visible": true
        }
      ],
      "type": "FRAME"
    },
    {
      "blendMode": "LINEAR_DODGE",
      "bounds": [
        73.5999984741211,
        896.9000244140625,
        28.80000114440918,
        45.0
      ],
      "characters": "LIVE\nFROM\nHELL",
      "fills": [
        {
          "color": [
            1.0,
            0.0,
            1.0,
            1.0
          ],
          "type": "SOLID",
          "visible": true
        },
        {
          "color": [
            0.0,
            0.0,
            0.0,
            0.0
          ],
          "type": "SOLID",
          "visible": true
        }
      ],
      "id": "stitch:0.1.1.1.0.0.0",
      "parentId": "stitch:0.1.1.1.0.0",
      "style": {
        "fontFamily": "Special Elite",
        "fontSize": 12.0,
        "fontWeight": 700,
        "lineHeightPercentFontSize": 125.0,
        "textAlignHorizontal": "CENTER"
      },
      "type": "TEXT"
    },
    {
      "bounds": [
        24.0,
        1187.800048828125,
        1318.0,
        120.0
      ],
      "id": "stitch:0.1.1.1.1",
      "parentId": "stitch:0.1.1.1",
      "type": "FRAME"
    },
    {
      "blendMode": "DIFFERENCE",
      "bounds": [
        24.0,
        1187.800048828125,
        324.0,
        120.0
      ],
      "characters": "NOISE\nCOMPLAINT",
      "effects": [
        {
          "color": [
            1.0,
            0.0,
            1.0,
            1.0
          ],
          "offset": [
            2.0,
            2.0
          ],
          "radius": 0.0,
          "type": "DROP_SHADOW",
          "visible": true
        }
      ],
      "fills": [
        {
          "color": [
            1.0,
            1.0,
            1.0,
            1.0
          ],
          "type": "SOLID",
          "visible": true
        },
        {
          "color": [
            0.0,
            0.0,
            0.0,
            0.0
          ],
          "type": "SOLID",
          "visible": true
        }
      ],
      "id": "stitch:0.1.1.1.1.0",
      "parentId": "stitch:0.1.1.1.1",
      "rotation": 1.0,
      "style": {
        "fontFamily": "Fugaz One",
        "fontSize": 60.0,
        "lineHeightPercentFontSize": 100.0
      },
      "type": "TEXT"
    },
    {
      "bounds": [
        24.0,
        1323.800048828125,
        216.0,
        51.0
      ],
      "id": "stitch:0.1.1.1.2",
      "layoutAlign": "MAX",
      "parentId": "stitch:0.1.1.1",
      "rotation": 1.0,
      "type": "FRAME"
    },
    {
      "bounds": [
        24.0,
        1323.800048828125,
        216.0,
        51.0
      ],
      "effects": [
        {
          "color": [
            0.800000011920929,
            1.0,
            0.0,
            1.0
          ],
          "offset": [
            6.0,
            6.0
          ],
          "radius": 0.0,
          "type": "DROP_SHADOW",
          "visible": true
        }
      ],
      "fills": [
        {
          "color": [
            0.0,
            0.0,
            0.0,
            1.0
          ],
          "type": "SOLID",
          "visible": true
        }
      ],
      "id": "stitch:0.1.1.1.2.0",
      "paddingBottom": 12.0,
      "paddingLeft": 24.0,
      "paddingRight": 24.0,
      "paddingTop": 12.0,
      "parentId": "stitch:0.1.1.1.2",
      "strokeAlign": "CENTER",
      "strokeWeight": 2.0,
      "strokes": [
        {
          "color": [
            1.0,
            1.0,
            1.0,
            1.0
          ],
          "type": "SOLID",
          "visible": true
        }
      ],
      "type": "FRAME"
    },
    {
      "bounds": [
        48.0,
        1335.800048828125,
        168.0,
        27.0
      ],
      "characters": "SIDE A: STATIC",
      "fills": [
        {
          "color": [
            0.800000011920929,
            1.0,
            0.0,
            1.0
          ],
          "type": "SOLID",
          "visible": true
        },
        {
          "color": [
            0.0,
            0.0,
            0.0,
            0.0
          ],
          "type": "SOLID",
          "visible": true
        }
      ],
      "id": "stitch:0.1.1.1.2.0.0",
      "parentId": "stitch:0.1.1.1.2.0",
      "style": {
        "fontFamily": "Special Elite",
        "fontSize": 20.0,
        "fontWeight": 700
      },
      "type": "TEXT"
    },
    {
      "bounds": [
        0.0,
        1502.800048828125,
        1366.0,
        751.4000244140625
      ],
      "clipsContent": true,
      "fills": [
        {
          "color": [
            0.0,
            0.0,
            0.0,
            1.0
          ],
          "type": "SOLID",
          "visible": true
        }
      ],
      "id": "stitch:0.1.2",
      "parentId": "stitch:0.1",
      "type": "FRAME"
    },
    {
      "bounds": [
        0.0,
        1502.800048828125,
        1366.0,
        751.4000244140625
      ],
      "fills": [
        {
          "color": [
            0.06666667014360428,
            0.0941176488995552,
            0.15294118225574493,
            1.0
          ],
          "type": "SOLID",
          "visible": true
        }
      ],
      "id": "stitch:0.1.2.0",
      "layoutPositioning": "ABSOLUTE",
      "parentId": "stitch:0.1.2",
      "type": "FRAME"
    },
    {
      "bounds": [
        0.0,
        1502.800048828125,
        1366.0,
        751.4000244140625
      ],
      "clipsContent": true,
      "fills": [
        {
          "imageFilter": {
            "contrast": 1.5,
            "grayscale": 1.0,
            "invert": 1.0
          },
          "imageRef": "filtered://composite://halftone/assets/lh3_googleusercontent_com_9013f386d7b1634f.png#f04ccace363fd396",
          "imageSourceRef": "composite://halftone/assets/lh3_googleusercontent_com_9013f386d7b1634f.png",
          "scaleMode": "FILL",
          "type": "IMAGE",
          "visible": true
        }
      ],
      "id": "stitch:0.1.2.0.0",
      "opacity": 0.6000000238418579,
      "parentId": "stitch:0.1.2.0",
      "type": "RECTANGLE"
    },
    {
      "bounds": [
        0.0,
        1502.800048828125,
        1366.0,
        751.4000244140625
      ],
      "fills": [
        {
          "end": [
            0.5,
            0.0
          ],
          "start": [
            0.5,
            1.0
          ],
          "stops": [
            {
              "color": [
                0.0,
                0.0,
                0.0,
                1.0
              ],
              "position": 0.0
            },
            {
              "color": [
                0.0,
                0.0,
                0.0,
                0.5
              ],
              "position": 0.5
            },
            {
              "color": [
                0.0,
                0.0,
                0.0,
                0.0
              ],
              "position": 1.0
            }
          ],
          "type": "GRADIENT_LINEAR",
          "visible": true
        }
      ],
      "id": "stitch:0.1.2.0.1",
      "layoutPositioning": "ABSOLUTE",
      "opacity": 0.8999999761581421,
      "parentId": "stitch:0.1.2.0",
      "type": "RECTANGLE"
    },
    {
      "bounds": [
        0.0,
        1502.800048828125,
        1366.0,
        751.4000244140625
      ],
      "counterAxisAlignItems": "CENTER",
      "id": "stitch:0.1.2.1",
      "layoutMode": "VERTICAL",
      "layoutPositioning": "ABSOLUTE",
      "paddingBottom": 24.0,
      "paddingLeft": 24.0,
      "paddingRight": 24.0,
      "paddingTop": 24.0,
      "parentId": "stitch:0.1.2",
      "primaryAxisAlignItems": "CENTER",
      "type": "FRAME"
    },
    {
      "bounds": [
        578.7999877929688,
        1781.550048828125,
        208.40000915527344,
        113.9000015258789
      ],
      "effects": [
        {
          "color": [
            1.0,
            1.0,
            1.0,
            1.0
          ],
          "offset": [
            4.0,
            4.0
          ],
          "radius": 0.0,
          "type": "DROP_SHADOW",
          "visible": true
        }
      ],
      "fills": [
        {
          "color": [
            0.0,
            0.0,
            0.0,
            1.0
          ],
          "type": "SOLID",
          "visible": true
        }
      ],
      "id": "stitch:0.1.2.1.0",
      "paddingBottom": 16.0,
      "paddingLeft": 16.0,
      "paddingRight": 16.0,
      "paddingTop": 16.0,
      "parentId": "stitch:0.1.2.1",
      "rotation": 2.0,
      "strokeAlign": "CENTER",
      "strokeWeight": 2.0,
      "strokes": [
        {
          "color": [
            1.0,
            1.0,
            1.0,
            1.0
          ],
          "type": "SOLID",
          "visible": true
        }
      ],
      "type": "FRAME"
    },
    {
      "bounds": [
        594.7999877929688,
        1797.550048828125,
        64.80000305175781,
        45.0
      ],
      "characters": "THE",
      "fills": [
        {
          "color": [
            1.0,
            1.0,
            1.0,
            1.0
          ],
          "type": "SOLID",
          "visible": true
        },
        {
          "color": [
            0.0,
            0.0,
            0.0,
            0.0
          ],
          "type": "SOLID",
          "visible": true
        }
      ],
      "id": "stitch:0.1.2.1.0.0",
      "parentId": "stitch:0.1.2.1.0",
      "style": {
        "fontFamily": "Sedgwick Ave Display",
        "fontSize": 36.0,
        "lineHeightPercentFontSize": 125.0,
        "textAlignHorizontal": "CENTER"
      },
      "type": "TEXT"
    },
    {
      "bounds": [
        594.7999877929688,
        1797.550048828125,
        151.20001220703125,
        45.0
      ],
      "characters": "REJECTS",
      "fills": [
        {
          "color": [
            0.800000011920929,
            1.0,
            0.0,
            1.0
          ],
          "type": "SOLID",
          "visible": true
        },
        {
          "color": [
            0.0,
            0.0,
            0.0,
            0.0
          ],
          "type": "SOLID",
          "visible": true
        }
      ],
      "id": "stitch:0.1.2.1.0.0.0",
      "parentId": "stitch:0.1.2.1.0.0",
      "style": {
        "fontFamily": "Sedgwick Ave Display",
        "fontSize": 36.0,
        "lineHeightPercentFontSize": 125.0,
        "textAlignHorizontal": "CENTER"
      },
      "type": "TEXT"
    },
    {
      "bounds": [
        594.7999877929688,
        1850.550048828125,
        1.0,
        2.0
      ],
      "fills": [
        {
          "color": [
            1.0,
            1.0,
            1.0,
            1.0
          ],
          "type": "SOLID",
          "visible": true
        }
      ],
      "id": "stitch:0.1.2.1.0.1",
      "parentId": "stitch:0.1.2.1.0",
      "type": "RECTANGLE"
    },
    {
      "bounds": [
        594.7999877929688,
        1860.550048828125,
        176.40000915527344,
        18.899999618530273
      ],
      "characters": "NEW DEMO TAPE OUT NOW",
      "fills": [
        {
          "color": [
            0.8196078538894653,
            0.8352941274642944,
            0.8588235378265381,
            1.0
          ],
          "type": "SOLID",
          "visible": true
        },
        {
          "color": [
            0.0,
            0.0,
            0.0,
            0.0
          ],
          "type": "SOLID",
          "visible": true
        }
      ],
      "id": "stitch:0.1.2.1.0.2",
      "parentId": "stitch:0.1.2.1.0",
      "style": {
        "fontFamily": "Special Elite",
        "fontSize": 14.0,
        "textAlignHorizontal": "CENTER"
      },
      "type": "TEXT"
    },
    {
      "bounds": [
        587.0,
        1927.4500732421875,
        192.0,
        48.0
      ],
      "id": "stitch:0.1.2.1.1",
      "parentId": "stitch:0.1.2.1",
      "rotation": -3.0,
      "type": "FRAME"
    },
    {
      "bounds": [
        587.0,
        1927.4500732421875,
        192.0,
        48.0
      ],
      "counterAxisAlignItems": "CENTER",
      "effects": [
        {
          "color": [
            0.0,
            0.0,
            0.0,
            0.800000011920929
          ],
          "offset": [
            0.0,
            1.0
          ],
          "radius": 3.0,
          "type": "DROP_SHADOW",
          "visible": true
        }
      ],
      "fills": [
        {
          "color": [
            0.11764705926179886,
            0.11764705926179886,
            0.11764705926179886,
            0.949999988079071
          ],
          "type": "SOLID",
          "visible": true
        }
      ],
      "id": "stitch:0.1.2.1.1.0",
      "layoutMode": "HORIZONTAL",
      "parentId": "stitch:0.1.2.1.1",
      "primaryAxisAlignItems": "CENTER",
      "rotation": -1.0,
      "strokeAlign": "CENTER",
      "strokeWeight": 1.0,
      "strokes": [
        {
          "color": [
            1.0,
            1.0,
            1.0,
            0.10000000149011612
          ],
          "type": "SOLID",
          "visible": true
        }
      ],
      "type": "FRAME"
    },
    {
      "bounds": [
        575.0,
        1911.4500732421875,
        92.0,
        27.0
      ],
      "characters": "Listen!",
      "fills": [
        {
          "color": [
            1.0,
            0.0,
            1.0,
            1.0
          ],
          "type": "SOLID",
          "visible": true
        },
        {
          "color": [
            0.0,
            0.0,
            0.0,
            1.0
          ],
          "type": "SOLID",
          "visible": true
        }
      ],
      "id": "stitch:0.1.2.1.1.0.0",
      "layoutPositioning": "ABSOLUTE",
      "paddingLeft": 4.0,
      "paddingRight": 4.0,
      "parentId": "stitch:0.1.2.1.1.0",
      "rotation": -12.0,
      "style": {
        "fontFamily": "Permanent Marker",
        "fontSize": 20.0,
        "textAlignHorizontal": "CENTER"
      },
      "type": "TEXT"
    },
    {
      "bounds": [
        615.7999877929688,
        1940.6500244140625,
        134.40000915527344,
        21.600000381469727
      ],
      "characters": "\"BASEMENT ROT\"",
      "fills": [
        {
          "color": [
            1.0,
            1.0,
            1.0,
            1.0
          ],
          "type": "SOLID",
          "visible": true
        },
        {
          "color": [
            0.0,
            0.0,
            0.0,
            0.0
          ],
          "type": "SOLID",
          "visible": true
        }
      ],
      "id": "stitch:0.1.2.1.1.0.1",
      "parentId": "stitch:0.1.2.1.1.0",
      "style": {
        "fontFamily": "Special Elite",
        "fontWeight": 700,
        "textAlignHorizontal": "CENTER"
      },
      "type": "TEXT"
    },
    {
      "bounds": [
        0.0,
        2254.2001953125,
        1366.0,
        256.0
      ],
      "counterAxisAlignItems": "CENTER",
      "fills": [
        {
          "color": [
            0.019607843831181526,
            0.019607843831181526,
            0.019607843831181526,
            1.0
          ],
          "type": "SOLID",
          "visible": true
        }
      ],
      "id": "stitch:0.1.3",
      "layoutMode": "VERTICAL",
      "paddingBottom": 32.0,
      "paddingLeft": 32.0,
      "paddingRight": 32.0,
      "paddingTop": 32.0,
      "parentId": "stitch:0.1",
      "primaryAxisAlignItems": "CENTER",
      "type": "FRAME"
    },
    {
      "bounds": [
        575.0,
        2357.400146484375,
        216.00001525878906,
        32.400001525878906
      ],
      "characters": "GO START A BAND",
      "fills": [
        {
          "color": [
            1.0,
            1.0,
            1.0,
            0.8999999761581421
          ],
          "type": "SOLID",
          "visible": true
        },
        {
          "color": [
            0.0,
            0.0,
            0.0,
            0.0
          ],
          "type": "SOLID",
          "visible": true
        }
      ],
      "id": "stitch:0.1.3.0",
      "parentId": "stitch:0.1.3",
      "rotation": -2.0,
      "style": {
        "fontFamily": "Permanent Marker",
        "fontSize": 24.0,
        "textAlignHorizontal": "CENTER"
      },
      "type": "TEXT"
    },
    {
      "bounds": [
        635.0,
        2389.80029296875,
        96.0,
        1.0
      ],
      "fills": [
        {
          "color": [
            1.0,
            1.0,
            1.0,
            0.30000001192092896
          ],
          "type": "SOLID",
          "visible": true
        }
      ],
      "id": "stitch:0.1.3.1",
      "parentId": "stitch:0.1.3",
      "type": "RECTANGLE"
    },
    {
      "bounds": [
        629.0,
        2390.80029296875,
        108.00000762939453,
        16.200000762939453
      ],
      "characters": "END OF THE ROLL",
      "fills": [
        {
          "color": [
            0.41960784792900085,
            0.4470588266849518,
            0.501960813999176,
            1.0
          ],
          "type": "SOLID",
          "visible": true
        },
        {
          "color": [
            0.0,
            0.0,
            0.0,
            0.0
          ],
          "type": "SOLID",
          "visible": true
        }
      ],
      "id": "stitch:0.1.3.2",
      "parentId": "stitch:0.1.3",
      "style": {
        "fontFamily": "Special Elite",
        "fontSize": 12.0,
        "textAlignHorizontal": "CENTER"
      },
      "type": "TEXT"
    },
    {
      "bounds": [
        0.0,
        2510.2001953125,
        1366.0,
        96.0
      ],
      "fills": [
        {
          "color": [
            0.019607843831181526,
            0.019607843831181526,
            0.019607843831181526,
            1.0
          ],
          "type": "SOLID",
          "visible": true
        }
      ],
      "id": "stitch:0.1.4",
      "parentId": "stitch:0.1",
      "type": "RECTANGLE"
    },
    {
      "bounds": [
        0.0,
        779.7999877929688,
        1366.0,
        104.19999694824219
      ],
      "id": "stitch:0.2",
      "layoutPositioning": "ABSOLUTE",
      "parentId": "stitch:0",
      "type": "FRAME"
    },
    {
      "bounds": [
        0.0,
        779.7999877929688,
        1366.0,
        104.19999694824219
      ],
      "counterAxisAlignItems": "MAX",
      "effects": [
        {
          "color": [
            0.0,
            0.0,
            0.0,
            0.800000011920929
          ],
          "offset": [
            0.0,
            -5.0
          ],
          "radius": 20.0,
          "type": "DROP_SHADOW",
          "visible": true
        }
      ],
      "fills": [
        {
          "color": [
            0.0,
            0.0,
            0.0,
            1.0
          ],
          "type": "SOLID",
          "visible": true
        }
      ],
      "id": "stitch:0.2.1",
      "layoutMode": "HORIZONTAL",
      "paddingBottom": 24.0,
      "paddingLeft": 24.0,
      "paddingRight": 24.0,
      "paddingTop": 8.0,
      "parentId": "stitch:0.2",
      "primaryAxisAlignItems": "SPACE_BETWEEN",
      "type": "FRAME"
    },
    {
      "blendMode": "OVERLAY",
      "bounds": [
        24.0,
        787.7999877929688,
        1318.0,
        72.19999694824219
      ],
      "fills": [
        {
          "imageRef": "procedural://noise/294f247ca9ff6ac5",
          "scaleMode": "TILE",
          "type": "IMAGE",
          "visible": true
        }
      ],
      "id": "stitch:0.2.1.0",
      "layoutPositioning": "ABSOLUTE",
      "opacity": 0.20000000298023224,
      "parentId": "stitch:0.2.1",
      "type": "RECTANGLE"
    },
    {
      "bounds": [
        24.0,
        787.7999877929688,
        439.3333435058594,
        72.19999694824219
      ],
      "counterAxisAlignItems": "CENTER",
      "id": "stitch:0.2.1.1",
      "itemSpacing": 4.0,
      "layoutGrow": 1.0,
      "layoutMode": "VERTICAL",
      "parentId": "stitch:0.2.1",
      "type": "FRAME"
    },
    {
      "bounds": [
        216.6666717529297,
        843.7999877929688,
        54.0,
        16.200000762939453
      ],
      "characters": "The Pit",
      "effects": [
        {
          "color": [
            0.0,
            0.0,
            0.0,
            1.0
          ],
          "offset": [
            0.0,
            2.0
          ],
          "radius": 0.0,
          "type": "DROP_SHADOW",
          "visible": true
        }
      ],
      "fills": [
        {
          "color": [
            0.800000011920929,
            1.0,
            0.0,
            1.0
          ],
          "type": "SOLID",
          "visible": true
        },
        {
          "color": [
            0.0,
            0.0,
            0.0,
            0.0
          ],
          "type": "SOLID",
          "visible": true
        }
      ],
      "id": "stitch:0.2.1.1.1",
      "opacity": 1.0,
      "parentId": "stitch:0.2.1.1",
      "rotation": -1.0,
      "style": {
        "fontFamily": "Fugaz One",
        "fontSize": 12.0,
        "letterSpacing": 0.6000000238418579,
        "textCase": "UPPER"
      },
      "type": "TEXT"
    },
    {
      "bounds": [
        217.6666717529297,
        779.7999877929688,
        52.0,
        52.0
      ],
      "cornerRadius": 2.0,
      "effects": [
        {
          "color": [
            1.0,
            1.0,
            1.0,
            1.0
          ],
          "offset": [
            2.0,
            2.0
          ],
          "radius": 0.0,
          "type": "DROP_SHADOW",
          "visible": true
        }
      ],
      "fills": [
        {
          "color": [
            0.800000011920929,
            1.0,
            0.0,
            1.0
          ],
          "type": "SOLID",
          "visible": true
        }
      ],
      "id": "stitch:0.2.1.1.0",
      "paddingBottom": 12.0,
      "paddingLeft": 12.0,
      "paddingRight": 12.0,
      "paddingTop": 12.0,
      "parentId": "stitch:0.2.1.1",
      "strokeAlign": "CENTER",
      "strokeWeight": 2.0,
      "strokes": [
        {
          "color": [
            0.0,
            0.0,
            0.0,
            1.0
          ],
          "type": "SOLID",
          "visible": true
        }
      ],
      "type": "FRAME"
    },
    {
      "bounds": [
        229.6666717529297,
        791.7999877929688,
        28.0,
        28.0
      ],
      "characters": "skull",
      "fills": [
        {
          "color": [
            0.0,
            0.0,
            0.0,
            1.0
          ],
          "type": "SOLID",
          "visible": true
        },
        {
          "color": [
            0.0,
            0.0,
            0.0,
            0.0
          ],
          "type": "SOLID",
          "visible": true
        }
      ],
      "id": "stitch:0.2.1.1.0.0",
      "parentId": "stitch:0.2.1.1.0",
      "style": {
        "fontFamily": "Material Symbols Outlined",
        "fontSize": 28.0,
        "fontStyle": "NORMAL",
        "fontWeight": 400,
        "letterSpacing": 0.0,
        "lineHeightPercentFontSize": 100.0
      },
      "type": "TEXT"
    },
    {
      "bounds": [
        463.3333435058594,
        812.0,
        439.3333435058594,
        48.0
      ],
      "counterAxisAlignItems": "CENTER",
      "id": "stitch:0.2.1.2",
      "itemSpacing": 4.0,
      "layoutGrow": 1.0,
      "layoutMode": "VERTICAL",
      "opacity": 0.699999988079071,
      "parentId": "stitch:0.2.1",
      "type": "FRAME"
    },
    {
      "bounds": [
        659.0,
        812.0,
        48.0,
        48.0
      ],
      "id": "stitch:0.2.1.2.0",
      "paddingBottom": 8.0,
      "paddingLeft": 8.0,
      "paddingRight": 8.0,
      "paddingTop": 8.0,
      "parentId": "stitch:0.2.1.2",
      "type": "FRAME"
    },
    {
      "bounds": [
        667.0,
        820.0,
        32.0,
        32.0
      ],
      "characters": "cable",
      "fills": [
        {
          "color": [
            1.0,
            1.0,
            1.0,
            1.0
          ],
          "type": "SOLID",
          "visible": true
        },
        {
          "color": [
            0.0,
            0.0,
            0.0,
            0.0
          ],
          "type": "SOLID",
          "visible": true
        }
      ],
      "id": "stitch:0.2.1.2.0.0",
      "parentId": "stitch:0.2.1.2.0",
      "style": {
        "fontFamily": "Material Symbols Outlined",
        "fontSize": 32.0,
        "fontStyle": "NORMAL",
        "fontWeight": 400,
        "letterSpacing": 0.0,
        "lineHeightPercentFontSize": 100.0
      },
      "type": "TEXT"
    },
    {
      "bounds": [
        902.6666870117188,
        812.0,
        439.3333435058594,
        48.0
      ],
      "counterAxisAlignItems": "CENTER",
      "id": "stitch:0.2.1.3",
      "itemSpacing": 4.0,
      "layoutGrow": 1.0,
      "layoutMode": "VERTICAL",
      "opacity": 0.699999988079071,
      "parentId": "stitch:0.2.1",
      "type": "FRAME"
    },
    {
      "bounds": [
        1098.3333740234375,
        812.0,
        48.0,
        48.0
      ],
      "id": "stitch:0.2.1.3.0",
      "paddingBottom": 8.0,
      "paddingLeft": 8.0,
      "paddingRight": 8.0,
      "paddingTop": 8.0,
      "parentId": "stitch:0.2.1.3",
      "type": "FRAME"
    },
    {
      "bounds": [
        1106.3333740234375,
        820.0,
        32.0,
        32.0
      ],
      "characters": "inventory_2",
      "fills": [
        {
          "color": [
            1.0,
            1.0,
            1.0,
            1.0
          ],
          "type": "SOLID",
          "visible": true
        },
        {
          "color": [
            0.0,
            0.0,
            0.0,
            0.0
          ],
          "type": "SOLID",
          "visible": true
        }
      ],
      "id": "stitch:0.2.1.3.0.0",
      "parentId": "stitch:0.2.1.3.0",
      "style": {
        "fontFamily": "Material Symbols Outlined",
        "fontSize": 32.0,
        "fontStyle": "NORMAL",
        "fontWeight": 400,
        "letterSpacing": 0.0,
        "lineHeightPercentFontSize": 100.0
      },
      "type": "TEXT"
    },
    {
      "bounds": [
        0.0,
        767.7999877929688,
        1366.0,
        16.0
      ],
      "fillGeometry": [
        "M 0 10 L 68.3 0 L 136.6 12 L 204.9 2 L 273.2 10 L 341.5 0 L 409.8 12 L 478.1 2 L 546.4 10 L 614.7 0 L 683 12 L 751.3 2 L 819.6 10 L 887.9 0 L 956.2 12 L 1024.5 2 L 1092.8 10 L 1161.1 0 L 1229.4 12 L 1297.7 2 L 1366 10 L 1366 16 L 0 16 Z"
      ],
      "fills": [
        {
          "color": [
            0.0,
            0.0,
            0.0,
            1.0
          ],
          "type": "SOLID",
          "visible": true
        }
      ],
      "id": "stitch:0.2.0",
      "layoutPositioning": "ABSOLUTE",
      "parentId": "stitch:0.2",
      "type": "RECTANGLE"
    },
    {
      "bounds": [
        0.0,
        0.0,
        1366.0,
        80.0
      ],
      "counterAxisAlignItems": "MIN",
      "id": "stitch:0.0",
      "layoutMode": "HORIZONTAL",
      "layoutPositioning": "ABSOLUTE",
      "paddingBottom": 16.0,
      "paddingLeft": 16.0,
      "paddingRight": 16.0,
      "paddingTop": 16.0,
      "parentId": "stitch:0",
      "primaryAxisAlignItems": "SPACE_BETWEEN",
      "type": "FRAME"
    },
    {
      "blendMode": "LINEAR_DODGE",
      "bounds": [
        16.0,
        16.0,
        94.0,
        35.0
      ],
      "effects": [
        {
          "color": [
            0.800000011920929,
            1.0,
            0.0,
            1.0
          ],
          "offset": [
            4.0,
            4.0
          ],
          "radius": 0.0,
          "type": "DROP_SHADOW",
          "visible": true
        }
      ],
      "fills": [
        {
          "color": [
            1.0,
            1.0,
            1.0,
            1.0
          ],
          "type": "SOLID",
          "visible": true
        }
      ],
      "id": "stitch:0.0.0",
      "paddingBottom": 4.0,
      "paddingLeft": 8.0,
      "paddingRight": 8.0,
      "paddingTop": 4.0,
      "parentId": "stitch:0.0",
      "rotation": -2.0,
      "type": "FRAME"
    },
    {
      "blendMode": "LINEAR_DODGE",
      "bounds": [
        24.0,
        20.0,
        78.0,
        27.0
      ],
      "characters": "THE PIT",
      "fills": [
        {
          "color": [
            0.0,
            0.0,
            0.0,
            1.0
          ],
          "type": "SOLID",
          "visible": true
        },
        {
          "color": [
            0.0,
            0.0,
            0.0,
            0.0
          ],
          "type": "SOLID",
          "visible": true
        }
      ],
      "id": "stitch:0.0.0.0",
      "parentId": "stitch:0.0.0",
      "style": {
        "fontFamily": "Fugaz One",
        "fontSize": 20.0,
        "letterSpacing": -1.0,
        "textCase": "UPPER"
      },
      "type": "TEXT"
    },
    {
      "bounds": [
        1302.0,
        16.0,
        48.0,
        48.0
      ],
      "counterAxisAlignItems": "CENTER",
      "id": "stitch:0.0.1",
      "itemSpacing": 4.0,
      "layoutMode": "HORIZONTAL",
      "parentId": "stitch:0.0",
      "type": "FRAME"
    },
    {
      "blendMode": "LINEAR_DODGE",
      "bounds": [
        1302.0,
        16.0,
        48.0,
        48.0
      ],
      "cornerRadius": 24.0,
      "counterAxisAlignItems": "CENTER",
      "effects": [
        {
          "color": [
            0.800000011920929,
            1.0,
            0.0,
            0.30000001192092896
          ],
          "offset": [
            0.0,
            0.0
          ],
          "radius": 10.0,
          "type": "DROP_SHADOW",
          "visible": true
        },
        {
          "radius": 8.0,
          "type": "BACKGROUND_BLUR",
          "visible": true
        }
      ],
      "fills": [
        {
          "color": [
            0.0,
            0.0,
            0.0,
            0.5
          ],
          "type": "SOLID",
          "visible": true
        }
      ],
      "id": "stitch:0.0.1.0",
      "layoutMode": "HORIZONTAL",
      "paddingBottom": 4.0,
      "paddingLeft": 4.0,
      "paddingRight": 4.0,
      "paddingTop": 4.0,
      "parentId": "stitch:0.0.1",
      "primaryAxisAlignItems": "CENTER",
      "rotation": 3.0,
      "strokeAlign": "CENTER",
      "strokeWeight": 2.0,
      "strokes": [
        {
          "color": [
            0.800000011920929,
            1.0,
            0.0,
            1.0
          ],
          "type": "SOLID",
          "visible": true
        }
      ],
      "type": "FRAME"
    },
    {
      "blendMode": "LINEAR_DODGE",
      "bounds": [
        1306.0,
        28.0,
        43.20000076293945,
        24.0
      ],
      "characters": "SEARCH\nHERE",
      "fills": [
        {
          "color": [
            0.800000011920929,
            1.0,
            0.0,
            1.0
          ],
          "type": "SOLID",
          "visible": true
        },
        {
          "color": [
            0.0,
            0.0,
            0.0,
            0.0
          ],
          "type": "SOLID",
          "visible": true
        }
      ],
      "id": "stitch:0.0.1.0.0",
      "parentId": "stitch:0.0.1.0",
      "style": {
        "fontFamily": "Permanent Marker",
        "fontSize": 12.0,
        "fontWeight": 700,
        "lineHeightPercentFontSize": 100.0,
        "textAlignHorizontal": "CENTER"
      },
      "type": "TEXT"
    },
    {
      "bounds": [
        0.0,
        0.0,
        1366.0,
        884.0
      ],
      "clipsContent": true,
      "id": "stitch:0.3",
      "layoutPositioning": "ABSOLUTE",
      "opacity": 0.20000000298023224,
      "parentId": "stitch:0",
      "type": "FRAME"
    },
    {
      "blendMode": "COLOR_DODGE",
      "bounds": [
        0.0,
        -32.0,
        1366.0,
        32.0
      ],
      "effects": [
        {
          "radius": 8.0,
          "type": "LAYER_BLUR",
          "visible": true
        }
      ],
      "fills": [
        {
          "end": [
            0.5,
            1.0
          ],
          "start": [
            0.5,
            0.0
          ],
          "stops": [
            {
              "color": [
                0.0,
                0.0,
                0.0,
                0.0
              ],
              "position": 0.0
            },
            {
              "color": [
                0.800000011920929,
                1.0,
                0.0,
                1.0
              ],
              "position": 0.5
            },
            {
              "color": [
                0.0,
                0.0,
                0.0,
                0.0
              ],
              "position": 1.0
            }
          ],
          "type": "GRADIENT_LINEAR",
          "visible": true
        }
      ],
      "id": "stitch:0.3.0",
      "layoutPositioning": "ABSOLUTE",
      "parentId": "stitch:0.3",
      "type": "RECTANGLE"
    }
  ]
}"#;

    pub fn document() -> Result<ImportedFigmaDocument, FigmaImportError> {
        import_figma_document(FIGMA_JSON)
    }

    pub fn runtime() -> Result<FigmaRuntime, FigmaRuntimeError> {
        FigmaRuntime::from_figma_json(FIGMA_JSON)
    }
}
