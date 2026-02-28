pub mod stitch_riot_scene_generated {

use arthropod::figma::{FigmaImportError, ImportedFigmaDocument, import_figma_document};
use arthropod::figma_runtime::{FigmaRuntime, FigmaRuntimeError};

pub const SOURCE_BYTES: usize = 39309;
pub const SOURCE_FNV64: u64 = 0x5808976b7fc2396a;
pub const NODE_COUNT: usize = 82;
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
      "blendMode": "LINEAR_DODGE",
      "bounds": [
        0.0,
        0.0,
        1366.0,
        115.0
      ],
      "counterAxisAlignItems": "MIN",
      "id": "stitch:0.0",
      "layoutMode": "VERTICAL",
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
      "bounds": [
        16.0,
        16.0,
        1334.0,
        35.0
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
      "type": "FRAME"
    },
    {
      "bounds": [
        24.0,
        20.0,
        78.4000015258789,
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
        }
      ],
      "id": "stitch:0.0.0.0",
      "parentId": "stitch:0.0.0",
      "style": {
        "fontFamily": "Fugaz One",
        "fontSize": 20.0,
        "textCase": "UPPER"
      },
      "type": "TEXT"
    },
    {
      "bounds": [
        16.0,
        51.0,
        1334.0,
        48.0
      ],
      "counterAxisAlignItems": "CENTER",
      "id": "stitch:0.0.1",
      "itemSpacing": 4.0,
      "layoutMode": "VERTICAL",
      "parentId": "stitch:0.0",
      "type": "FRAME"
    },
    {
      "bounds": [
        659.0,
        51.0,
        48.0,
        48.0
      ],
      "cornerRadius": 24.0,
      "counterAxisAlignItems": "CENTER",
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
      "layoutMode": "VERTICAL",
      "paddingBottom": 4.0,
      "paddingLeft": 4.0,
      "paddingRight": 4.0,
      "paddingTop": 4.0,
      "parentId": "stitch:0.0.1",
      "primaryAxisAlignItems": "CENTER",
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
      "bounds": [
        656.1199951171875,
        59.0,
        53.76000213623047,
        32.0
      ],
      "characters": "SEARCH\nHERE",
      "fills": [
        {
          "color": [
            0.9490196108818054,
            0.9490196108818054,
            0.9490196108818054,
            1.0
          ],
          "type": "SOLID",
          "visible": true
        }
      ],
      "id": "stitch:0.0.1.0.0",
      "parentId": "stitch:0.0.1.0",
      "style": {
        "fontFamily": "Space Grotesk",
        "fontWeight": 700,
        "lineHeightPercentFontSize": 100.0,
        "textAlignHorizontal": "CENTER"
      },
      "type": "TEXT"
    },
    {
      "bounds": [
        656.1199951171875,
        59.0,
        53.76000213623047,
        40.0
      ],
      "id": "stitch:0.0.1.0.0.0",
      "parentId": "stitch:0.0.1.0.0",
      "type": "RECTANGLE"
    },
    {
      "bounds": [
        0.0,
        0.0,
        1366.0,
        2606.2001953125
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
        884.0
      ],
      "fills": [
        {
          "imageRef": "assets/lh3_googleusercontent_com_fbf53e3f8caee692.png",
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
      "blendMode": "SCREEN",
      "bounds": [
        8.0,
        104.0,
        1318.0,
        80.0
      ],
      "id": "stitch:0.1.0.1.0",
      "layoutPositioning": "ABSOLUTE",
      "opacity": 0.8999999761581421,
      "parentId": "stitch:0.1.0.1",
      "type": "FRAME"
    },
    {
      "bounds": [
        8.0,
        104.0,
        80.0,
        80.0
      ],
      "counterAxisAlignItems": "CENTER",
      "id": "stitch:0.1.0.1.0.0",
      "layoutMode": "VERTICAL",
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
      "bounds": [
        25.600000381469727,
        124.0,
        44.79999923706055,
        40.0
      ],
      "characters": "TODAY\nONLY",
      "fills": [
        {
          "color": [
            0.9490196108818054,
            0.9490196108818054,
            0.9490196108818054,
            1.0
          ],
          "type": "SOLID",
          "visible": true
        }
      ],
      "id": "stitch:0.1.0.1.0.0.0",
      "parentId": "stitch:0.1.0.1.0.0",
      "style": {
        "fontFamily": "Space Grotesk",
        "fontWeight": 700,
        "lineHeightPercentFontSize": 125.0,
        "textAlignHorizontal": "CENTER"
      },
      "type": "TEXT"
    },
    {
      "bounds": [
        25.600000381469727,
        124.0,
        44.79999923706055,
        40.0
      ],
      "id": "stitch:0.1.0.1.0.0.0.0",
      "parentId": "stitch:0.1.0.1.0.0.0",
      "type": "RECTANGLE"
    },
    {
      "bounds": [
        24.0,
        508.10003662109375,
        1318.0,
        32.0
      ],
      "id": "stitch:0.1.0.1.1",
      "parentId": "stitch:0.1.0.1",
      "type": "FRAME"
    },
    {
      "bounds": [
        24.0,
        508.10003662109375,
        44.79999923706055,
        32.0
      ],
      "characters": "SCUM\nFUCKS",
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
      "id": "stitch:0.1.0.1.1.0",
      "parentId": "stitch:0.1.0.1.1",
      "style": {
        "fontFamily": "Sedgwick Ave Display",
        "lineHeightPercentFontSize": 100.0
      },
      "type": "TEXT"
    },
    {
      "bounds": [
        24.0,
        508.10003662109375,
        51.84000015258789,
        21.600000381469727
      ],
      "characters": "SCUM",
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
      "style": {
        "fontFamily": "Sedgwick Ave Display",
        "lineHeightPercentFontSize": 100.0
      },
      "type": "TEXT"
    },
    {
      "bounds": [
        24.0,
        529.7000122070312,
        44.79999923706055,
        40.0
      ],
      "id": "stitch:0.1.0.1.1.0.1",
      "parentId": "stitch:0.1.0.1.1.0",
      "type": "RECTANGLE"
    },
    {
      "bounds": [
        56.0,
        569.7000122070312,
        60.79999923706055,
        21.600000381469727
      ],
      "characters": "FUCKS",
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
            1.0
          ],
          "type": "SOLID",
          "visible": true
        }
      ],
      "id": "stitch:0.1.0.1.1.0.2",
      "paddingLeft": 8.0,
      "paddingRight": 8.0,
      "parentId": "stitch:0.1.0.1.1.0",
      "style": {
        "fontFamily": "Sedgwick Ave Display",
        "lineHeightPercentFontSize": 100.0
      },
      "type": "TEXT"
    },
    {
      "bounds": [
        24.0,
        564.1000366210938,
        1318.0,
        75.30000305175781
      ],
      "id": "stitch:0.1.0.1.2",
      "layoutAlign": "MIN",
      "parentId": "stitch:0.1.0.1",
      "type": "FRAME"
    },
    {
      "bounds": [
        24.0,
        564.1000366210938,
        1318.0,
        75.30000305175781
      ],
      "id": "stitch:0.1.0.1.2.0",
      "paddingBottom": 8.0,
      "paddingLeft": 16.0,
      "paddingRight": 16.0,
      "paddingTop": 8.0,
      "parentId": "stitch:0.1.0.1.2",
      "strokeAlign": "CENTER",
      "strokeWeight": 1.0,
      "strokes": [
        {
          "color": [
            0.21568627655506134,
            0.2549019753932953,
            0.3176470696926117,
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
        627.0,
        572.1000366210938,
        112.0,
        27.0
      ],
      "characters": "graphic_eq",
      "fills": [
        {
          "color": [
            0.9490196108818054,
            0.9490196108818054,
            0.9490196108818054,
            1.0
          ],
          "type": "SOLID",
          "visible": true
        }
      ],
      "id": "stitch:0.1.0.1.2.0.0",
      "parentId": "stitch:0.1.0.1.2.0",
      "style": {
        "fontFamily": "Space Grotesk",
        "fontSize": 20.0
      },
      "type": "TEXT"
    },
    {
      "bounds": [
        622.52001953125,
        607.1000366210938,
        120.95999908447266,
        24.30000114440918
      ],
      "characters": "\"RAT POISON\"",
      "fills": [
        {
          "color": [
            0.9490196108818054,
            0.9490196108818054,
            0.9490196108818054,
            1.0
          ],
          "type": "SOLID",
          "visible": true
        }
      ],
      "id": "stitch:0.1.0.1.2.0.1",
      "parentId": "stitch:0.1.0.1.2.0",
      "style": {
        "fontFamily": "Space Grotesk",
        "fontSize": 18.0,
        "fontWeight": 700,
        "textCase": "UPPER"
      },
      "type": "TEXT"
    },
    {
      "bounds": [
        0.0,
        503.4000244140625,
        1318.0,
        56.0
      ],
      "id": "stitch:0.1.0.1.3",
      "layoutPositioning": "ABSOLUTE",
      "parentId": "stitch:0.1.0.1",
      "type": "FRAME"
    },
    {
      "bounds": [
        0.0,
        503.4000244140625,
        1318.0,
        56.0
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
      "id": "stitch:0.1.0.1.3.0",
      "paddingBottom": 8.0,
      "paddingLeft": 16.0,
      "paddingRight": 16.0,
      "paddingTop": 8.0,
      "parentId": "stitch:0.1.0.1.3",
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
      "type": "RECTANGLE"
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
        884.0
      ],
      "fills": [
        {
          "imageRef": "assets/lh3_googleusercontent_com_9d79993df332b436.png",
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
      "blendMode": "LINEAR_DODGE",
      "bounds": [
        40.0,
        871.4000244140625,
        1318.0,
        96.0
      ],
      "id": "stitch:0.1.1.1.0",
      "layoutPositioning": "ABSOLUTE",
      "opacity": 0.8999999761581421,
      "parentId": "stitch:0.1.1.1",
      "type": "FRAME"
    },
    {
      "bounds": [
        40.0,
        871.4000244140625,
        96.0,
        96.0
      ],
      "cornerRadius": 48.0,
      "counterAxisAlignItems": "CENTER",
      "id": "stitch:0.1.1.1.0.0",
      "layoutMode": "VERTICAL",
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
      "bounds": [
        70.08000183105469,
        889.4000244140625,
        35.84000015258789,
        60.0
      ],
      "characters": "LIVE\nFROM\nHELL",
      "fills": [
        {
          "color": [
            0.9490196108818054,
            0.9490196108818054,
            0.9490196108818054,
            1.0
          ],
          "type": "SOLID",
          "visible": true
        }
      ],
      "id": "stitch:0.1.1.1.0.0.0",
      "parentId": "stitch:0.1.1.1.0.0",
      "style": {
        "fontFamily": "Space Grotesk",
        "fontWeight": 700,
        "lineHeightPercentFontSize": 125.0,
        "textAlignHorizontal": "CENTER"
      },
      "type": "TEXT"
    },
    {
      "bounds": [
        70.08000183105469,
        889.4000244140625,
        35.84000015258789,
        40.0
      ],
      "id": "stitch:0.1.1.1.0.0.0.0",
      "parentId": "stitch:0.1.1.1.0.0.0",
      "type": "RECTANGLE"
    },
    {
      "bounds": [
        70.08000183105469,
        929.4000244140625,
        35.84000015258789,
        40.0
      ],
      "id": "stitch:0.1.1.1.0.0.0.1",
      "parentId": "stitch:0.1.1.1.0.0.0",
      "type": "RECTANGLE"
    },
    {
      "bounds": [
        24.0,
        1275.800048828125,
        1318.0,
        32.0
      ],
      "id": "stitch:0.1.1.1.1",
      "parentId": "stitch:0.1.1.1",
      "type": "FRAME"
    },
    {
      "blendMode": "DIFFERENCE",
      "bounds": [
        24.0,
        1275.800048828125,
        80.63999938964844,
        32.0
      ],
      "characters": "NOISE\nCOMPLAINT",
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
      "id": "stitch:0.1.1.1.1.0",
      "parentId": "stitch:0.1.1.1.1",
      "style": {
        "fontFamily": "Fugaz One",
        "lineHeightPercentFontSize": 100.0
      },
      "type": "TEXT"
    },
    {
      "bounds": [
        24.0,
        1275.800048828125,
        80.63999938964844,
        40.0
      ],
      "id": "stitch:0.1.1.1.1.0.0",
      "parentId": "stitch:0.1.1.1.1.0",
      "type": "RECTANGLE"
    },
    {
      "bounds": [
        24.0,
        1323.800048828125,
        1318.0,
        51.0
      ],
      "id": "stitch:0.1.1.1.2",
      "layoutAlign": "MAX",
      "parentId": "stitch:0.1.1.1",
      "type": "FRAME"
    },
    {
      "bounds": [
        24.0,
        1323.800048828125,
        1318.0,
        51.0
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
        156.8000030517578,
        27.0
      ],
      "characters": "SIDE A: STATIC",
      "fills": [
        {
          "color": [
            0.9490196108818054,
            0.9490196108818054,
            0.9490196108818054,
            1.0
          ],
          "type": "SOLID",
          "visible": true
        }
      ],
      "id": "stitch:0.1.1.1.2.0.0",
      "parentId": "stitch:0.1.1.1.2.0",
      "style": {
        "fontFamily": "Space Grotesk",
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
        884.0
      ],
      "fills": [
        {
          "imageRef": "assets/lh3_googleusercontent_com_9013f386d7b1634f.png",
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
        24.0,
        1792.7000732421875,
        1318.0,
        91.5999984741211
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
        40.0,
        1808.7000732421875,
        98.55999755859375,
        20.0
      ],
      "characters": "THE REJECTS",
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
      "id": "stitch:0.1.2.1.0.0",
      "parentId": "stitch:0.1.2.1.0",
      "style": {
        "fontFamily": "Sedgwick Ave Display",
        "lineHeightPercentFontSize": 125.0,
        "textAlignHorizontal": "CENTER"
      },
      "type": "TEXT"
    },
    {
      "bounds": [
        40.0,
        1808.7000732421875,
        62.720001220703125,
        21.600000381469727
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
        }
      ],
      "id": "stitch:0.1.2.1.0.0.0",
      "parentId": "stitch:0.1.2.1.0.0",
      "style": {
        "fontFamily": "Sedgwick Ave Display",
        "lineHeightPercentFontSize": 125.0,
        "textAlignHorizontal": "CENTER"
      },
      "type": "TEXT"
    },
    {
      "bounds": [
        40.0,
        1836.7000732421875,
        1286.0,
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
        40.0,
        1846.7000732421875,
        188.16000366210938,
        21.600000381469727
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
        }
      ],
      "id": "stitch:0.1.2.1.0.2",
      "parentId": "stitch:0.1.2.1.0",
      "style": {
        "fontFamily": "Special Elite",
        "textAlignHorizontal": "CENTER"
      },
      "type": "TEXT"
    },
    {
      "bounds": [
        24.0,
        1916.300048828125,
        1318.0,
        48.0
      ],
      "id": "stitch:0.1.2.1.1",
      "parentId": "stitch:0.1.2.1",
      "type": "FRAME"
    },
    {
      "bounds": [
        24.0,
        1916.300048828125,
        192.0,
        48.0
      ],
      "counterAxisAlignItems": "CENTER",
      "id": "stitch:0.1.2.1.1.0",
      "layoutMode": "VERTICAL",
      "parentId": "stitch:0.1.2.1.1",
      "primaryAxisAlignItems": "CENTER",
      "strokeAlign": "CENTER",
      "strokeWeight": 1.0,
      "strokes": [
        {
          "color": [
            0.29411765933036804,
            0.3333333432674408,
            0.38823530077934265,
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
        24.0,
        1916.300048828125,
        86.4000015258789,
        27.0
      ],
      "characters": "Listen!",
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
            0.9490196108818054,
            0.9490196108818054,
            0.9490196108818054,
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
      "style": {
        "fontFamily": "Permanent Marker",
        "fontSize": 20.0,
        "textAlignHorizontal": "CENTER"
      },
      "type": "TEXT"
    },
    {
      "bounds": [
        57.279998779296875,
        1929.5,
        125.44000244140625,
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
        }
      ],
      "id": "stitch:0.1.2.1.1.0.1",
      "parentId": "stitch:0.1.2.1.1.0",
      "style": {
        "fontFamily": "Space Grotesk",
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
        615.7999877929688,
        2360.10009765625,
        134.39999389648438,
        21.600000381469727
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
        }
      ],
      "id": "stitch:0.1.3.0",
      "parentId": "stitch:0.1.3",
      "style": {
        "fontFamily": "Permanent Marker",
        "textAlignHorizontal": "CENTER"
      },
      "type": "TEXT"
    },
    {
      "bounds": [
        635.0,
        2381.7001953125,
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
        615.7999877929688,
        2382.7001953125,
        134.39999389648438,
        21.600000381469727
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
        }
      ],
      "id": "stitch:0.1.3.2",
      "parentId": "stitch:0.1.3",
      "style": {
        "fontFamily": "Special Elite",
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
        651.5999755859375,
        1366.0,
        232.39999389648438
      ],
      "id": "stitch:0.2",
      "layoutPositioning": "ABSOLUTE",
      "parentId": "stitch:0",
      "type": "FRAME"
    },
    {
      "bounds": [
        0.0,
        651.5999755859375,
        1366.0,
        16.0
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
        651.5999755859375,
        1366.0,
        232.39999389648438
      ],
      "counterAxisAlignItems": "MAX",
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
      "layoutMode": "VERTICAL",
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
        659.5999755859375,
        1318.0,
        200.39999389648438
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
        659.5999755859375,
        1318.0,
        82.0
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
        24.0,
        659.5999755859375,
        1318.0,
        61.79999923706055
      ],
      "cornerRadius": 2.0,
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
        36.0,
        671.5999755859375,
        78.4000015258789,
        37.79999923706055
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
        }
      ],
      "id": "stitch:0.2.1.1.0.0",
      "parentId": "stitch:0.2.1.1.0",
      "style": {
        "fontFamily": "Space Grotesk",
        "fontSize": 28.0
      },
      "type": "TEXT"
    },
    {
      "bounds": [
        659.47998046875,
        725.3999633789062,
        47.040000915527344,
        16.200000762939453
      ],
      "characters": "The Pit",
      "fills": [
        {
          "color": [
            0.9490196108818054,
            0.9490196108818054,
            0.9490196108818054,
            1.0
          ],
          "type": "SOLID",
          "visible": true
        }
      ],
      "id": "stitch:0.2.1.1.1",
      "opacity": 1.0,
      "parentId": "stitch:0.2.1.1",
      "style": {
        "fontFamily": "Fugaz One",
        "fontSize": 12.0,
        "textCase": "UPPER"
      },
      "type": "TEXT"
    },
    {
      "bounds": [
        24.0,
        741.5999755859375,
        1318.0,
        59.20000076293945
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
        24.0,
        741.5999755859375,
        1318.0,
        59.20000076293945
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
        32.0,
        749.5999755859375,
        89.5999984741211,
        43.20000076293945
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
        }
      ],
      "id": "stitch:0.2.1.2.0.0",
      "parentId": "stitch:0.2.1.2.0",
      "style": {
        "fontFamily": "Space Grotesk",
        "fontSize": 32.0
      },
      "type": "TEXT"
    },
    {
      "bounds": [
        24.0,
        741.5999755859375,
        26.880001068115234,
        16.200000762939453
      ],
      "characters": "Deck",
      "fills": [
        {
          "color": [
            0.9490196108818054,
            0.9490196108818054,
            0.9490196108818054,
            1.0
          ],
          "type": "SOLID",
          "visible": true
        }
      ],
      "id": "stitch:0.2.1.2.1",
      "layoutPositioning": "ABSOLUTE",
      "parentId": "stitch:0.2.1.2",
      "style": {
        "fontFamily": "Fugaz One",
        "fontSize": 12.0,
        "textCase": "UPPER"
      },
      "type": "TEXT"
    },
    {
      "bounds": [
        24.0,
        800.7999877929688,
        1318.0,
        59.20000076293945
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
        24.0,
        800.7999877929688,
        1318.0,
        59.20000076293945
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
        32.0,
        808.7999877929688,
        197.1199951171875,
        43.20000076293945
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
        }
      ],
      "id": "stitch:0.2.1.3.0.0",
      "parentId": "stitch:0.2.1.3.0",
      "style": {
        "fontFamily": "Space Grotesk",
        "fontSize": 32.0
      },
      "type": "TEXT"
    },
    {
      "bounds": [
        24.0,
        800.7999877929688,
        33.599998474121094,
        16.200000762939453
      ],
      "characters": "Stash",
      "fills": [
        {
          "color": [
            0.9490196108818054,
            0.9490196108818054,
            0.9490196108818054,
            1.0
          ],
          "type": "SOLID",
          "visible": true
        }
      ],
      "id": "stitch:0.2.1.3.1",
      "layoutPositioning": "ABSOLUTE",
      "parentId": "stitch:0.2.1.3",
      "style": {
        "fontFamily": "Fugaz One",
        "fontSize": 12.0,
        "textCase": "UPPER"
      },
      "type": "TEXT"
    },
    {
      "blendMode": "COLOR_DODGE",
      "bounds": [
        0.0,
        0.0,
        1366.0,
        884.0
      ],
      "id": "stitch:0.3",
      "layoutPositioning": "ABSOLUTE",
      "opacity": 0.20000000298023224,
      "parentId": "stitch:0",
      "type": "FRAME"
    },
    {
      "bounds": [
        0.0,
        0.0,
        1366.0,
        32.0
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
