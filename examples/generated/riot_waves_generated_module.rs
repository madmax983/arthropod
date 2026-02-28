pub mod riot_waves_generated {

    use arthropod::figma::{FigmaImportError, ImportedFigmaDocument, import_figma_document};
    use arthropod::figma_runtime::{FigmaRuntime, FigmaRuntimeError};

    pub const SOURCE_BYTES: usize = 282143;
    pub const SOURCE_FNV64: u64 = 0x832215d45002fd4b;
    pub const NODE_COUNT: usize = 127;
    pub const PROTOTYPE_EDGE_COUNT: usize = 0;

    pub const FIGMA_JSON: &str = r##"[
    {
        "name": "Indie Artist Spotify App",
        "type": "FRAME",
        "x": 0,
        "y": 0,
        "width": 1029,
        "height": 812,
        "fills": [
            {
                "type": "SOLID",
                "visible": true,
                "opacity": 1,
                "blendMode": "NORMAL",
                "color": {
                    "r": 0,
                    "g": 0,
                    "b": 0
                },
                "boundVariables": {}
            }
        ],
        "children": [
            {
                "name": "Container",
                "type": "FRAME",
                "x": 0,
                "y": 0,
                "width": 1029,
                "height": 812,
                "children": [
                    {
                        "name": "aside",
                        "type": "FRAME",
                        "x": 0,
                        "y": 0,
                        "width": 256,
                        "height": 812,
                        "fills": [
                            {
                                "type": "SOLID",
                                "visible": true,
                                "opacity": 1,
                                "blendMode": "NORMAL",
                                "color": {
                                    "r": 0,
                                    "g": 0,
                                    "b": 0
                                },
                                "boundVariables": {}
                            }
                        ],
                        "strokes": [
                            {
                                "type": "SOLID",
                                "opacity": 0.20000000298023224,
                                "blendMode": "NORMAL",
                                "color": "#f6339a"
                            }
                        ],
                        "children": [
                            {
                                "name": "div",
                                "type": "FRAME",
                                "x": 0,
                                "y": 0,
                                "width": 255,
                                "height": 101,
                                "strokes": [
                                    {
                                        "type": "SOLID",
                                        "opacity": 0.20000000298023224,
                                        "blendMode": "NORMAL",
                                        "color": "#f6339a"
                                    }
                                ],
                                "children": [
                                    {
                                        "name": "Container",
                                        "type": "FRAME",
                                        "x": 24,
                                        "y": 24,
                                        "width": 207,
                                        "height": 32,
                                        "children": [
                                            {
                                                "type": "VECTOR",
                                                "x": 0,
                                                "y": 0,
                                                "width": 32,
                                                "height": 32
                                            },
                                            {
                                                "name": "h1",
                                                "type": "FRAME",
                                                "x": 40,
                                                "y": 0,
                                                "width": 140.078125,
                                                "height": 32,
                                                "children": [
                                                    {
                                                        "name": "RIOTWAVES",
                                                        "type": "TEXT",
                                                        "x": 0,
                                                        "y": 0,
                                                        "width": 144,
                                                        "height": 32,
                                                        "characters": "RIOTWAVES",
                                                        "fontSize": 24,
                                                        "fontName": {
                                                            "family": "Inter",
                                                            "style": "Black"
                                                        },
                                                        "textAlignHorizontal": "LEFT",
                                                        "textAlignVertical": "TOP",
                                                        "lineHeight": {
                                                            "unit": "PIXELS",
                                                            "value": 32
                                                        },
                                                        "tailwind": {
                                                            "width": "w-144",
                                                            "height": "h-32",
                                                            "font-size": "text-24"
                                                        }
                                                    }
                                                ],
                                                "layoutMode": "HORIZONTAL",
                                                "primaryAxisSizingMode": "FIXED",
                                                "counterAxisSizingMode": "FIXED",
                                                "primaryAxisAlignItems": "MIN",
                                                "counterAxisAlignItems": "MIN",
                                                "tailwind": {
                                                    "width": "w-140",
                                                    "height": "h-32",
                                                    "margin-left": "ml-40",
                                                    "flex": "flex",
                                                    "justify-content": "justify-start",
                                                    "align-items": "items-start"
                                                }
                                            }
                                        ],
                                        "layoutMode": "HORIZONTAL",
                                        "primaryAxisSizingMode": "FIXED",
                                        "counterAxisSizingMode": "FIXED",
                                        "primaryAxisAlignItems": "MIN",
                                        "counterAxisAlignItems": "CENTER",
                                        "itemSpacing": 8,
                                        "tailwind": {
                                            "width": "w-207",
                                            "height": "h-32",
                                            "margin-left": "ml-24",
                                            "flex": "flex gap-8",
                                            "justify-content": "justify-start",
                                            "align-items": "items-center"
                                        }
                                    },
                                    {
                                        "name": "p",
                                        "type": "FRAME",
                                        "x": 24,
                                        "y": 60,
                                        "width": 207,
                                        "height": 16,
                                        "children": [
                                            {
                                                "name": "FOR THE UNDERGROUND",
                                                "type": "TEXT",
                                                "x": 0,
                                                "y": 0,
                                                "width": 126,
                                                "height": 16,
                                                "fills": [
                                                    {
                                                        "type": "SOLID",
                                                        "visible": true,
                                                        "opacity": 1,
                                                        "blendMode": "NORMAL",
                                                        "color": {
                                                            "r": 0.4156862795352936,
                                                            "g": 0.4470588266849518,
                                                            "b": 0.5098039507865906
                                                        },
                                                        "boundVariables": {}
                                                    }
                                                ],
                                                "characters": "FOR THE UNDERGROUND",
                                                "fontSize": 12,
                                                "fontName": {
                                                    "family": "Consolas",
                                                    "style": "Regular"
                                                },
                                                "textAlignHorizontal": "LEFT",
                                                "textAlignVertical": "TOP",
                                                "lineHeight": {
                                                    "unit": "PIXELS",
                                                    "value": 16
                                                },
                                                "tailwind": {
                                                    "width": "w-126",
                                                    "height": "h-16",
                                                    "font-size": "text-12"
                                                }
                                            }
                                        ],
                                        "layoutMode": "NONE",
                                        "primaryAxisSizingMode": "AUTO",
                                        "counterAxisSizingMode": "FIXED",
                                        "primaryAxisAlignItems": "MIN",
                                        "counterAxisAlignItems": "MIN",
                                        "tailwind": {
                                            "width": "w-207",
                                            "height": "h-16",
                                            "margin-left": "ml-24",
                                            "flex": "flex",
                                            "justify-content": "justify-start",
                                            "align-items": "items-start"
                                        }
                                    }
                                ],
                                "layoutMode": "VERTICAL",
                                "primaryAxisSizingMode": "FIXED",
                                "counterAxisSizingMode": "FIXED",
                                "primaryAxisAlignItems": "MIN",
                                "counterAxisAlignItems": "MIN",
                                "paddingTop": 24,
                                "paddingRight": 24,
                                "paddingBottom": 1,
                                "paddingLeft": 24,
                                "itemSpacing": 4,
                                "tailwind": {
                                    "width": "w-255",
                                    "height": "h-101",
                                    "padding": "px-24 pt-24 pb-1",
                                    "flex": "flex flex-col gap-4",
                                    "justify-content": "justify-start",
                                    "align-items": "items-start"
                                }
                            },
                            {
                                "name": "nav",
                                "type": "FRAME",
                                "x": 0,
                                "y": 101,
                                "width": 255,
                                "height": 194,
                                "children": [
                                    {
                                        "name": "Link",
                                        "type": "FRAME",
                                        "x": 16,
                                        "y": 16,
                                        "width": 223,
                                        "height": 50,
                                        "fills": [
                                            {
                                                "type": "SOLID",
                                                "visible": true,
                                                "opacity": 0.20000000298023224,
                                                "blendMode": "NORMAL",
                                                "color": {
                                                    "r": 0.9647058844566345,
                                                    "g": 0.20000000298023224,
                                                    "b": 0.6039215922355652
                                                },
                                                "boundVariables": {}
                                            }
                                        ],
                                        "strokes": [
                                            {
                                                "type": "SOLID",
                                                "opacity": 0.4000000059604645,
                                                "blendMode": "NORMAL",
                                                "color": "#f6339a"
                                            }
                                        ],
                                        "cornerRadius": 10,
                                        "children": [
                                            {
                                                "type": "VECTOR",
                                                "x": 17,
                                                "y": 15,
                                                "width": 20,
                                                "height": 20
                                            },
                                            {
                                                "name": "span",
                                                "type": "FRAME",
                                                "x": 49,
                                                "y": 13,
                                                "width": 43.9921875,
                                                "height": 24,
                                                "children": [
                                                    {
                                                        "name": "Home",
                                                        "type": "TEXT",
                                                        "x": 0,
                                                        "y": -2,
                                                        "width": 46,
                                                        "height": 24,
                                                        "fills": [
                                                            {
                                                                "type": "SOLID",
                                                                "visible": true,
                                                                "opacity": 1,
                                                                "blendMode": "NORMAL",
                                                                "color": {
                                                                    "r": 0.9647058844566345,
                                                                    "g": 0.20000000298023224,
                                                                    "b": 0.6039215922355652
                                                                },
                                                                "boundVariables": {}
                                                            }
                                                        ],
                                                        "characters": "Home",
                                                        "fontSize": 16,
                                                        "fontName": {
                                                            "family": "Inter",
                                                            "style": "Semi Bold"
                                                        },
                                                        "textAlignHorizontal": "LEFT",
                                                        "textAlignVertical": "TOP",
                                                        "lineHeight": {
                                                            "unit": "PIXELS",
                                                            "value": 24
                                                        },
                                                        "tailwind": {
                                                            "width": "w-46",
                                                            "height": "h-24",
                                                            "font-size": "text-16"
                                                        }
                                                    }
                                                ],
                                                "layoutMode": "NONE",
                                                "primaryAxisSizingMode": "AUTO",
                                                "counterAxisSizingMode": "FIXED",
                                                "primaryAxisAlignItems": "MIN",
                                                "counterAxisAlignItems": "MIN",
                                                "tailwind": {
                                                    "width": "w-44",
                                                    "height": "h-24",
                                                    "margin-left": "ml-49",
                                                    "flex": "flex",
                                                    "justify-content": "justify-start",
                                                    "align-items": "items-start"
                                                }
                                            }
                                        ],
                                        "layoutMode": "HORIZONTAL",
                                        "primaryAxisSizingMode": "FIXED",
                                        "counterAxisSizingMode": "FIXED",
                                        "primaryAxisAlignItems": "MIN",
                                        "counterAxisAlignItems": "CENTER",
                                        "paddingLeft": 16,
                                        "itemSpacing": 12,
                                        "tailwind": {
                                            "width": "w-223",
                                            "height": "h-50",
                                            "margin-left": "ml-16",
                                            "padding": "pl-16",
                                            "flex": "flex gap-12",
                                            "justify-content": "justify-start",
                                            "align-items": "items-center"
                                        }
                                    },
                                    {
                                        "name": "Link",
                                        "type": "FRAME",
                                        "x": 16,
                                        "y": 74,
                                        "width": 223,
                                        "height": 48,
                                        "cornerRadius": 10,
                                        "children": [
                                            {
                                                "type": "VECTOR",
                                                "x": 16,
                                                "y": 14,
                                                "width": 20,
                                                "height": 20
                                            },
                                            {
                                                "name": "span",
                                                "type": "FRAME",
                                                "x": 48,
                                                "y": 12,
                                                "width": 52.5,
                                                "height": 24,
                                                "children": [
                                                    {
                                                        "name": "Browse",
                                                        "type": "TEXT",
                                                        "x": 0,
                                                        "y": -2,
                                                        "width": 58,
                                                        "height": 24,
                                                        "fills": [
                                                            {
                                                                "type": "SOLID",
                                                                "visible": true,
                                                                "opacity": 1,
                                                                "blendMode": "NORMAL",
                                                                "color": {
                                                                    "r": 0.6000000238418579,
                                                                    "g": 0.6313725709915161,
                                                                    "b": 0.686274528503418
                                                                },
                                                                "boundVariables": {}
                                                            }
                                                        ],
                                                        "characters": "Browse",
                                                        "fontSize": 16,
                                                        "fontName": {
                                                            "family": "Inter",
                                                            "style": "Semi Bold"
                                                        },
                                                        "textAlignHorizontal": "LEFT",
                                                        "textAlignVertical": "TOP",
                                                        "lineHeight": {
                                                            "unit": "PIXELS",
                                                            "value": 24
                                                        },
                                                        "tailwind": {
                                                            "width": "w-58",
                                                            "height": "h-24",
                                                            "font-size": "text-16"
                                                        }
                                                    }
                                                ],
                                                "layoutMode": "NONE",
                                                "primaryAxisSizingMode": "AUTO",
                                                "counterAxisSizingMode": "FIXED",
                                                "primaryAxisAlignItems": "MIN",
                                                "counterAxisAlignItems": "MIN",
                                                "tailwind": {
                                                    "width": "w-53",
                                                    "height": "h-24",
                                                    "margin-left": "ml-48",
                                                    "flex": "flex",
                                                    "justify-content": "justify-start",
                                                    "align-items": "items-start"
                                                }
                                            }
                                        ],
                                        "layoutMode": "HORIZONTAL",
                                        "primaryAxisSizingMode": "FIXED",
                                        "counterAxisSizingMode": "FIXED",
                                        "primaryAxisAlignItems": "MIN",
                                        "counterAxisAlignItems": "CENTER",
                                        "paddingLeft": 16,
                                        "itemSpacing": 12,
                                        "tailwind": {
                                            "width": "w-223",
                                            "height": "h-48",
                                            "margin-left": "ml-16",
                                            "padding": "pl-16",
                                            "flex": "flex gap-12",
                                            "justify-content": "justify-start",
                                            "align-items": "items-center"
                                        }
                                    },
                                    {
                                        "name": "Link",
                                        "type": "FRAME",
                                        "x": 16,
                                        "y": 130,
                                        "width": 223,
                                        "height": 48,
                                        "cornerRadius": 10,
                                        "children": [
                                            {
                                                "type": "VECTOR",
                                                "x": 16,
                                                "y": 14,
                                                "width": 20,
                                                "height": 20
                                            },
                                            {
                                                "name": "span",
                                                "type": "FRAME",
                                                "x": 48,
                                                "y": 12,
                                                "width": 41.6875,
                                                "height": 24,
                                                "children": [
                                                    {
                                                        "name": "Radio",
                                                        "type": "TEXT",
                                                        "x": 0,
                                                        "y": -2,
                                                        "width": 44,
                                                        "height": 24,
                                                        "fills": [
                                                            {
                                                                "type": "SOLID",
                                                                "visible": true,
                                                                "opacity": 1,
                                                                "blendMode": "NORMAL",
                                                                "color": {
                                                                    "r": 0.6000000238418579,
                                                                    "g": 0.6313725709915161,
                                                                    "b": 0.686274528503418
                                                                },
                                                                "boundVariables": {}
                                                            }
                                                        ],
                                                        "characters": "Radio",
                                                        "fontSize": 16,
                                                        "fontName": {
                                                            "family": "Inter",
                                                            "style": "Semi Bold"
                                                        },
                                                        "textAlignHorizontal": "LEFT",
                                                        "textAlignVertical": "TOP",
                                                        "lineHeight": {
                                                            "unit": "PIXELS",
                                                            "value": 24
                                                        },
                                                        "tailwind": {
                                                            "width": "w-44",
                                                            "height": "h-24",
                                                            "font-size": "text-16"
                                                        }
                                                    }
                                                ],
                                                "layoutMode": "NONE",
                                                "primaryAxisSizingMode": "AUTO",
                                                "counterAxisSizingMode": "FIXED",
                                                "primaryAxisAlignItems": "MIN",
                                                "counterAxisAlignItems": "MIN",
                                                "tailwind": {
                                                    "width": "w-42",
                                                    "height": "h-24",
                                                    "margin-left": "ml-48",
                                                    "flex": "flex",
                                                    "justify-content": "justify-start",
                                                    "align-items": "items-start"
                                                }
                                            }
                                        ],
                                        "layoutMode": "HORIZONTAL",
                                        "primaryAxisSizingMode": "FIXED",
                                        "counterAxisSizingMode": "FIXED",
                                        "primaryAxisAlignItems": "MIN",
                                        "counterAxisAlignItems": "CENTER",
                                        "paddingLeft": 16,
                                        "itemSpacing": 12,
                                        "tailwind": {
                                            "width": "w-223",
                                            "height": "h-48",
                                            "margin-left": "ml-16",
                                            "padding": "pl-16",
                                            "flex": "flex gap-12",
                                            "justify-content": "justify-start",
                                            "align-items": "items-center"
                                        }
                                    }
                                ],
                                "layoutMode": "VERTICAL",
                                "primaryAxisSizingMode": "FIXED",
                                "counterAxisSizingMode": "FIXED",
                                "primaryAxisAlignItems": "MIN",
                                "counterAxisAlignItems": "MIN",
                                "paddingTop": 16,
                                "paddingRight": 16,
                                "paddingLeft": 16,
                                "itemSpacing": 8,
                                "tailwind": {
                                    "width": "w-255",
                                    "height": "h-194",
                                    "padding": "px-16 pt-16",
                                    "flex": "flex flex-col gap-8",
                                    "justify-content": "justify-start",
                                    "align-items": "items-start"
                                }
                            },
                            {
                                "name": "div",
                                "type": "FRAME",
                                "x": 0,
                                "y": 295,
                                "width": 255,
                                "height": 452,
                                "children": [
                                    {
                                        "name": "h3",
                                        "type": "FRAME",
                                        "x": 16,
                                        "y": 16,
                                        "width": 223,
                                        "height": 16,
                                        "children": [
                                            {
                                                "name": "Playlists",
                                                "type": "TEXT",
                                                "x": 16,
                                                "y": 0,
                                                "width": 191,
                                                "height": 16,
                                                "fills": [
                                                    {
                                                        "type": "SOLID",
                                                        "visible": true,
                                                        "opacity": 1,
                                                        "blendMode": "NORMAL",
                                                        "color": {
                                                            "r": 0.4156862795352936,
                                                            "g": 0.4470588266849518,
                                                            "b": 0.5098039507865906
                                                        },
                                                        "boundVariables": {}
                                                    }
                                                ],
                                                "characters": "Playlists",
                                                "fontSize": 12,
                                                "fontName": {
                                                    "family": "Inter",
                                                    "style": "Black"
                                                },
                                                "textAlignHorizontal": "LEFT",
                                                "textAlignVertical": "TOP",
                                                "lineHeight": {
                                                    "unit": "PIXELS",
                                                    "value": 16
                                                },
                                                "tailwind": {
                                                    "width": "w-191",
                                                    "height": "h-16",
                                                    "margin-left": "ml-16",
                                                    "font-size": "text-12"
                                                }
                                            }
                                        ],
                                        "layoutMode": "HORIZONTAL",
                                        "primaryAxisSizingMode": "FIXED",
                                        "counterAxisSizingMode": "FIXED",
                                        "primaryAxisAlignItems": "MIN",
                                        "counterAxisAlignItems": "MIN",
                                        "paddingRight": 16,
                                        "paddingLeft": 16,
                                        "tailwind": {
                                            "width": "w-223",
                                            "height": "h-16",
                                            "margin-left": "ml-16",
                                            "padding": "px-16",
                                            "flex": "flex",
                                            "justify-content": "justify-start",
                                            "align-items": "items-start"
                                        }
                                    },
                                    {
                                        "name": "Container",
                                        "type": "FRAME",
                                        "x": 16,
                                        "y": 44,
                                        "width": 223,
                                        "height": 116,
                                        "children": [
                                            {
                                                "name": "Link",
                                                "type": "FRAME",
                                                "x": 0,
                                                "y": 0,
                                                "width": 223,
                                                "height": 36,
                                                "cornerRadius": 10,
                                                "children": [
                                                    {
                                                        "type": "VECTOR",
                                                        "x": 16,
                                                        "y": 10,
                                                        "width": 16,
                                                        "height": 16
                                                    },
                                                    {
                                                        "name": "span",
                                                        "type": "FRAME",
                                                        "x": 44,
                                                        "y": 8,
                                                        "width": 93.28125,
                                                        "height": 20,
                                                        "children": [
                                                            {
                                                                "name": "Punk Essentials",
                                                                "type": "TEXT",
                                                                "x": 0,
                                                                "y": -0.5,
                                                                "width": 104,
                                                                "height": 20,
                                                                "fills": [
                                                                    {
                                                                        "type": "SOLID",
                                                                        "visible": true,
                                                                        "opacity": 1,
                                                                        "blendMode": "NORMAL",
                                                                        "color": {
                                                                            "r": 0.6000000238418579,
                                                                            "g": 0.6313725709915161,
                                                                            "b": 0.686274528503418
                                                                        },
                                                                        "boundVariables": {}
                                                                    }
                                                                ],
                                                                "characters": "Punk Essentials",
                                                                "fontSize": 14,
                                                                "fontName": {
                                                                    "family": "Inter",
                                                                    "style": "Regular"
                                                                },
                                                                "textAlignHorizontal": "LEFT",
                                                                "textAlignVertical": "TOP",
                                                                "lineHeight": {
                                                                    "unit": "PIXELS",
                                                                    "value": 20
                                                                },
                                                                "tailwind": {
                                                                    "width": "w-104",
                                                                    "height": "h-20",
                                                                    "font-size": "text-14"
                                                                }
                                                            }
                                                        ],
                                                        "layoutMode": "NONE",
                                                        "primaryAxisSizingMode": "AUTO",
                                                        "counterAxisSizingMode": "FIXED",
                                                        "primaryAxisAlignItems": "MIN",
                                                        "counterAxisAlignItems": "MIN",
                                                        "tailwind": {
                                                            "width": "w-93",
                                                            "height": "h-20",
                                                            "margin-left": "ml-44",
                                                            "flex": "flex",
                                                            "justify-content": "justify-start",
                                                            "align-items": "items-start"
                                                        }
                                                    }
                                                ],
                                                "layoutMode": "HORIZONTAL",
                                                "primaryAxisSizingMode": "FIXED",
                                                "counterAxisSizingMode": "FIXED",
                                                "primaryAxisAlignItems": "MIN",
                                                "counterAxisAlignItems": "CENTER",
                                                "paddingLeft": 16,
                                                "itemSpacing": 12,
                                                "tailwind": {
                                                    "width": "w-223",
                                                    "height": "h-36",
                                                    "padding": "pl-16",
                                                    "flex": "flex gap-12",
                                                    "justify-content": "justify-start",
                                                    "align-items": "items-center"
                                                }
                                            },
                                            {
                                                "name": "Link",
                                                "type": "FRAME",
                                                "x": 0,
                                                "y": 40,
                                                "width": 223,
                                                "height": 36,
                                                "cornerRadius": 10,
                                                "children": [
                                                    {
                                                        "type": "VECTOR",
                                                        "x": 16,
                                                        "y": 10,
                                                        "width": 16,
                                                        "height": 16
                                                    },
                                                    {
                                                        "name": "span",
                                                        "type": "FRAME",
                                                        "x": 44,
                                                        "y": 8,
                                                        "width": 68.0234375,
                                                        "height": 20,
                                                        "children": [
                                                            {
                                                                "name": "Indie Vibes",
                                                                "type": "TEXT",
                                                                "x": 0,
                                                                "y": -0.5,
                                                                "width": 73,
                                                                "height": 20,
                                                                "fills": [
                                                                    {
                                                                        "type": "SOLID",
                                                                        "visible": true,
                                                                        "opacity": 1,
                                                                        "blendMode": "NORMAL",
                                                                        "color": {
                                                                            "r": 0.6000000238418579,
                                                                            "g": 0.6313725709915161,
                                                                            "b": 0.686274528503418
                                                                        },
                                                                        "boundVariables": {}
                                                                    }
                                                                ],
                                                                "characters": "Indie Vibes",
                                                                "fontSize": 14,
                                                                "fontName": {
                                                                    "family": "Inter",
                                                                    "style": "Regular"
                                                                },
                                                                "textAlignHorizontal": "LEFT",
                                                                "textAlignVertical": "TOP",
                                                                "lineHeight": {
                                                                    "unit": "PIXELS",
                                                                    "value": 20
                                                                },
                                                                "tailwind": {
                                                                    "width": "w-73",
                                                                    "height": "h-20",
                                                                    "font-size": "text-14"
                                                                }
                                                            }
                                                        ],
                                                        "layoutMode": "NONE",
                                                        "primaryAxisSizingMode": "AUTO",
                                                        "counterAxisSizingMode": "FIXED",
                                                        "primaryAxisAlignItems": "MIN",
                                                        "counterAxisAlignItems": "MIN",
                                                        "tailwind": {
                                                            "width": "w-68",
                                                            "height": "h-20",
                                                            "margin-left": "ml-44",
                                                            "flex": "flex",
                                                            "justify-content": "justify-start",
                                                            "align-items": "items-start"
                                                        }
                                                    }
                                                ],
                                                "layoutMode": "HORIZONTAL",
                                                "primaryAxisSizingMode": "FIXED",
                                                "counterAxisSizingMode": "FIXED",
                                                "primaryAxisAlignItems": "MIN",
                                                "counterAxisAlignItems": "CENTER",
                                                "paddingLeft": 16,
                                                "itemSpacing": 12,
                                                "tailwind": {
                                                    "width": "w-223",
                                                    "height": "h-36",
                                                    "padding": "pl-16",
                                                    "flex": "flex gap-12",
                                                    "justify-content": "justify-start",
                                                    "align-items": "items-center"
                                                }
                                            },
                                            {
                                                "name": "Link",
                                                "type": "FRAME",
                                                "x": 0,
                                                "y": 80,
                                                "width": 223,
                                                "height": 36,
                                                "cornerRadius": 10,
                                                "children": [
                                                    {
                                                        "type": "VECTOR",
                                                        "x": 16,
                                                        "y": 10,
                                                        "width": 16,
                                                        "height": 16
                                                    },
                                                    {
                                                        "name": "span",
                                                        "type": "FRAME",
                                                        "x": 44,
                                                        "y": 8,
                                                        "width": 71.6640625,
                                                        "height": 20,
                                                        "children": [
                                                            {
                                                                "name": "Raw Energy",
                                                                "type": "TEXT",
                                                                "x": 0,
                                                                "y": -0.5,
                                                                "width": 79,
                                                                "height": 20,
                                                                "fills": [
                                                                    {
                                                                        "type": "SOLID",
                                                                        "visible": true,
                                                                        "opacity": 1,
                                                                        "blendMode": "NORMAL",
                                                                        "color": {
                                                                            "r": 0.6000000238418579,
                                                                            "g": 0.6313725709915161,
                                                                            "b": 0.686274528503418
                                                                        },
                                                                        "boundVariables": {}
                                                                    }
                                                                ],
                                                                "characters": "Raw Energy",
                                                                "fontSize": 14,
                                                                "fontName": {
                                                                    "family": "Inter",
                                                                    "style": "Regular"
                                                                },
                                                                "textAlignHorizontal": "LEFT",
                                                                "textAlignVertical": "TOP",
                                                                "lineHeight": {
                                                                    "unit": "PIXELS",
                                                                    "value": 20
                                                                },
                                                                "tailwind": {
                                                                    "width": "w-79",
                                                                    "height": "h-20",
                                                                    "font-size": "text-14"
                                                                }
                                                            }
                                                        ],
                                                        "layoutMode": "NONE",
                                                        "primaryAxisSizingMode": "AUTO",
                                                        "counterAxisSizingMode": "FIXED",
                                                        "primaryAxisAlignItems": "MIN",
                                                        "counterAxisAlignItems": "MIN",
                                                        "tailwind": {
                                                            "width": "w-72",
                                                            "height": "h-20",
                                                            "margin-left": "ml-44",
                                                            "flex": "flex",
                                                            "justify-content": "justify-start",
                                                            "align-items": "items-start"
                                                        }
                                                    }
                                                ],
                                                "layoutMode": "HORIZONTAL",
                                                "primaryAxisSizingMode": "FIXED",
                                                "counterAxisSizingMode": "FIXED",
                                                "primaryAxisAlignItems": "MIN",
                                                "counterAxisAlignItems": "CENTER",
                                                "paddingLeft": 16,
                                                "itemSpacing": 12,
                                                "tailwind": {
                                                    "width": "w-223",
                                                    "height": "h-36",
                                                    "padding": "pl-16",
                                                    "flex": "flex gap-12",
                                                    "justify-content": "justify-start",
                                                    "align-items": "items-center"
                                                }
                                            }
                                        ],
                                        "layoutMode": "VERTICAL",
                                        "primaryAxisSizingMode": "FIXED",
                                        "counterAxisSizingMode": "FIXED",
                                        "primaryAxisAlignItems": "MIN",
                                        "counterAxisAlignItems": "MIN",
                                        "itemSpacing": 4,
                                        "tailwind": {
                                            "width": "w-223",
                                            "height": "h-116",
                                            "margin-left": "ml-16",
                                            "flex": "flex flex-col gap-4",
                                            "justify-content": "justify-start",
                                            "align-items": "items-start"
                                        }
                                    }
                                ],
                                "layoutMode": "VERTICAL",
                                "primaryAxisSizingMode": "FIXED",
                                "counterAxisSizingMode": "FIXED",
                                "primaryAxisAlignItems": "MIN",
                                "counterAxisAlignItems": "MIN",
                                "paddingTop": 16,
                                "paddingRight": 16,
                                "paddingLeft": 16,
                                "itemSpacing": 12,
                                "tailwind": {
                                    "width": "w-255",
                                    "height": "h-452",
                                    "padding": "px-16 pt-16",
                                    "flex": "flex flex-col gap-12",
                                    "justify-content": "justify-start",
                                    "align-items": "items-start"
                                }
                            },
                            {
                                "name": "div",
                                "type": "FRAME",
                                "x": 0,
                                "y": 747,
                                "width": 255,
                                "height": 65,
                                "strokes": [
                                    {
                                        "type": "SOLID",
                                        "opacity": 0.20000000298023224,
                                        "blendMode": "NORMAL",
                                        "color": "#f6339a"
                                    }
                                ],
                                "children": [
                                    {
                                        "name": "p",
                                        "type": "FRAME",
                                        "x": 16,
                                        "y": 17,
                                        "width": 223,
                                        "height": 32,
                                        "children": [
                                            {
                                                "name": "SUPPORTING INDIE ARTISTS SINCE 2026",
                                                "type": "TEXT",
                                                "x": 0,
                                                "y": 0,
                                                "width": 198,
                                                "height": 32,
                                                "fills": [
                                                    {
                                                        "type": "SOLID",
                                                        "visible": true,
                                                        "opacity": 1,
                                                        "blendMode": "NORMAL",
                                                        "color": {
                                                            "r": 0.29019609093666077,
                                                            "g": 0.3333333432674408,
                                                            "b": 0.3960784375667572
                                                        },
                                                        "boundVariables": {}
                                                    }
                                                ],
                                                "characters": "SUPPORTING INDIE ARTISTS SINCE 2026",
                                                "fontSize": 12,
                                                "fontName": {
                                                    "family": "Consolas",
                                                    "style": "Regular"
                                                },
                                                "textAlignHorizontal": "LEFT",
                                                "textAlignVertical": "TOP",
                                                "lineHeight": {
                                                    "unit": "PIXELS",
                                                    "value": 16
                                                },
                                                "tailwind": {
                                                    "width": "w-198",
                                                    "height": "h-32",
                                                    "font-size": "text-12"
                                                }
                                            }
                                        ],
                                        "layoutMode": "NONE",
                                        "primaryAxisSizingMode": "AUTO",
                                        "counterAxisSizingMode": "FIXED",
                                        "primaryAxisAlignItems": "MIN",
                                        "counterAxisAlignItems": "MIN",
                                        "tailwind": {
                                            "width": "w-223",
                                            "height": "h-32",
                                            "margin-left": "ml-16",
                                            "flex": "flex",
                                            "justify-content": "justify-start",
                                            "align-items": "items-start"
                                        }
                                    }
                                ],
                                "layoutMode": "VERTICAL",
                                "primaryAxisSizingMode": "FIXED",
                                "counterAxisSizingMode": "FIXED",
                                "primaryAxisAlignItems": "MIN",
                                "counterAxisAlignItems": "MIN",
                                "paddingTop": 17,
                                "paddingRight": 16,
                                "paddingLeft": 16,
                                "tailwind": {
                                    "width": "w-255",
                                    "height": "h-65",
                                    "padding": "px-16 pt-17",
                                    "flex": "flex flex-col",
                                    "justify-content": "justify-start",
                                    "align-items": "items-start"
                                }
                            }
                        ],
                        "layoutMode": "VERTICAL",
                        "primaryAxisSizingMode": "FIXED",
                        "counterAxisSizingMode": "FIXED",
                        "primaryAxisAlignItems": "MIN",
                        "counterAxisAlignItems": "MIN",
                        "tailwind": {
                            "width": "w-256",
                            "height": "h-812",
                            "flex": "flex flex-col",
                            "justify-content": "justify-start",
                            "align-items": "items-start"
                        }
                    },
                    {
                        "name": "main",
                        "type": "FRAME",
                        "x": 256,
                        "y": 0,
                        "width": 773,
                        "height": 812,
                        "fills": [
                            {
                                "type": "GRADIENT_LINEAR",
                                "visible": true,
                                "opacity": 1,
                                "blendMode": "NORMAL",
                                "gradientStops": [
                                    {
                                        "color": {
                                            "r": 0,
                                            "g": 0,
                                            "b": 0,
                                            "a": 1
                                        },
                                        "position": 0,
                                        "boundVariables": {}
                                    },
                                    {
                                        "color": {
                                            "r": 0.03529411926865578,
                                            "g": 0.03529411926865578,
                                            "b": 0.04313725605607033,
                                            "a": 1
                                        },
                                        "position": 0.5,
                                        "boundVariables": {}
                                    },
                                    {
                                        "color": {
                                            "r": 0,
                                            "g": 0,
                                            "b": 0,
                                            "a": 1
                                        },
                                        "position": 1,
                                        "boundVariables": {}
                                    }
                                ],
                                "gradientTransform": [
                                    [
                                        0.5,
                                        0.5,
                                        0
                                    ],
                                    [
                                        -0.25,
                                        0.25,
                                        0.5
                                    ]
                                ]
                            }
                        ],
                        "children": [
                            {
                                "name": "div",
                                "type": "FRAME",
                                "x": 0,
                                "y": 0,
                                "width": 758,
                                "height": 1282.3359375,
                                "children": [
                                    {
                                        "name": "section",
                                        "type": "FRAME",
                                        "x": 32,
                                        "y": 32,
                                        "width": 694,
                                        "height": 320,
                                        "cornerRadius": 14,
                                        "children": [
                                            {
                                                "type": "VECTOR",
                                                "x": 0,
                                                "y": 0,
                                                "width": 694,
                                                "height": 320
                                            },
                                            {
                                                "type": "VECTOR",
                                                "x": 0,
                                                "y": 0,
                                                "width": 694,
                                                "height": 320
                                            },
                                            {
                                                "name": "div",
                                                "type": "FRAME",
                                                "x": 0,
                                                "y": 0,
                                                "width": 694,
                                                "height": 320,
                                                "fills": [
                                                    {
                                                        "type": "GRADIENT_LINEAR",
                                                        "visible": true,
                                                        "opacity": 1,
                                                        "blendMode": "NORMAL",
                                                        "gradientStops": [
                                                            {
                                                                "color": {
                                                                    "r": 0,
                                                                    "g": 0,
                                                                    "b": 0,
                                                                    "a": 1
                                                                },
                                                                "position": 0,
                                                                "boundVariables": {}
                                                            },
                                                            {
                                                                "color": {
                                                                    "r": 0,
                                                                    "g": 0,
                                                                    "b": 0,
                                                                    "a": 0.5
                                                                },
                                                                "position": 0.5,
                                                                "boundVariables": {}
                                                            },
                                                            {
                                                                "color": {
                                                                    "r": 0,
                                                                    "g": 0,
                                                                    "b": 0,
                                                                    "a": 0
                                                                },
                                                                "position": 1,
                                                                "boundVariables": {}
                                                            }
                                                        ],
                                                        "gradientTransform": [
                                                            [
                                                                0,
                                                                -1,
                                                                1
                                                            ],
                                                            [
                                                                0.5,
                                                                0,
                                                                0.25
                                                            ]
                                                        ]
                                                    }
                                                ],
                                                "children": [
                                                    {
                                                        "name": "h2",
                                                        "type": "FRAME",
                                                        "x": 48,
                                                        "y": 128,
                                                        "width": 598,
                                                        "height": 120,
                                                        "children": [
                                                            {
                                                                "name": "UNLEASH THE",
                                                                "type": "TEXT",
                                                                "x": 0,
                                                                "y": -4,
                                                                "width": 410,
                                                                "height": 60,
                                                                "fills": [
                                                                    {
                                                                        "type": "SOLID",
                                                                        "visible": true,
                                                                        "opacity": 1,
                                                                        "blendMode": "NORMAL",
                                                                        "color": {
                                                                            "r": 1,
                                                                            "g": 1,
                                                                            "b": 1
                                                                        },
                                                                        "boundVariables": {}
                                                                    }
                                                                ],
                                                                "characters": "UNLEASH THE",
                                                                "fontSize": 60,
                                                                "fontName": {
                                                                    "family": "Inter",
                                                                    "style": "Black"
                                                                },
                                                                "textAlignHorizontal": "LEFT",
                                                                "textAlignVertical": "TOP",
                                                                "lineHeight": {
                                                                    "unit": "PIXELS",
                                                                    "value": 60
                                                                },
                                                                "tailwind": {
                                                                    "width": "w-410",
                                                                    "height": "h-60",
                                                                    "font-size": "text-60"
                                                                }
                                                            },
                                                            {
                                                                "name": "span",
                                                                "type": "FRAME",
                                                                "x": 0,
                                                                "y": 50,
                                                                "width": 465.3203125,
                                                                "height": 79.5,
                                                                "children": [
                                                                    {
                                                                        "name": "UNDERGROUND",
                                                                        "type": "TEXT",
                                                                        "x": 0,
                                                                        "y": 0,
                                                                        "width": 458,
                                                                        "height": 60,
                                                                        "fills": [
                                                                            {
                                                                                "type": "GRADIENT_LINEAR",
                                                                                "visible": true,
                                                                                "opacity": 1,
                                                                                "blendMode": "NORMAL",
                                                                                "gradientStops": [
                                                                                    {
                                                                                        "color": {
                                                                                            "r": 0.9647058844566345,
                                                                                            "g": 0.20000000298023224,
                                                                                            "b": 0.6039215922355652,
                                                                                            "a": 1
                                                                                        },
                                                                                        "position": 0,
                                                                                        "boundVariables": {}
                                                                                    },
                                                                                    {
                                                                                        "color": {
                                                                                            "r": 0,
                                                                                            "g": 0.7215686440467834,
                                                                                            "b": 0.8588235378265381,
                                                                                            "a": 1
                                                                                        },
                                                                                        "position": 1,
                                                                                        "boundVariables": {}
                                                                                    }
                                                                                ],
                                                                                "gradientTransform": [
                                                                                    [
                                                                                        1,
                                                                                        0,
                                                                                        0
                                                                                    ],
                                                                                    [
                                                                                        0,
                                                                                        0.5,
                                                                                        0.25
                                                                                    ]
                                                                                ]
                                                                            },
                                                                            {
                                                                                "type": "SOLID",
                                                                                "visible": true,
                                                                                "opacity": 0,
                                                                                "blendMode": "NORMAL",
                                                                                "color": {
                                                                                    "r": 0,
                                                                                    "g": 0,
                                                                                    "b": 0
                                                                                },
                                                                                "boundVariables": {}
                                                                            }
                                                                        ],
                                                                        "characters": "UNDERGROUND",
                                                                        "fontSize": 60,
                                                                        "fontName": {
                                                                            "family": "Inter",
                                                                            "style": "Black"
                                                                        },
                                                                        "textAlignHorizontal": "LEFT",
                                                                        "textAlignVertical": "TOP",
                                                                        "lineHeight": {
                                                                            "unit": "PIXELS",
                                                                            "value": 60
                                                                        },
                                                                        "tailwind": {
                                                                            "width": "w-458",
                                                                            "height": "h-60",
                                                                            "font-size": "text-60"
                                                                        }
                                                                    }
                                                                ],
                                                                "layoutMode": "HORIZONTAL",
                                                                "primaryAxisSizingMode": "FIXED",
                                                                "counterAxisSizingMode": "FIXED",
                                                                "primaryAxisAlignItems": "MIN",
                                                                "counterAxisAlignItems": "MIN",
                                                                "tailwind": {
                                                                    "width": "w-465",
                                                                    "height": "h-80",
                                                                    "flex": "flex",
                                                                    "justify-content": "justify-start",
                                                                    "align-items": "items-start"
                                                                }
                                                            }
                                                        ],
                                                        "layoutMode": "NONE",
                                                        "primaryAxisSizingMode": "AUTO",
                                                        "counterAxisSizingMode": "FIXED",
                                                        "primaryAxisAlignItems": "MIN",
                                                        "counterAxisAlignItems": "MIN",
                                                        "tailwind": {
                                                            "width": "w-598",
                                                            "height": "h-120",
                                                            "margin-left": "ml-48",
                                                            "flex": "flex",
                                                            "justify-content": "justify-start",
                                                            "align-items": "items-start"
                                                        }
                                                    },
                                                    {
                                                        "name": "p",
                                                        "type": "FRAME",
                                                        "x": 48,
                                                        "y": 264,
                                                        "width": 598,
                                                        "height": 56,
                                                        "children": [
                                                            {
                                                                "name": "Discover raw, unfiltered music from indie artists ",
                                                                "type": "TEXT",
                                                                "x": 0,
                                                                "y": -1,
                                                                "width": 561,
                                                                "height": 56,
                                                                "fills": [
                                                                    {
                                                                        "type": "SOLID",
                                                                        "visible": true,
                                                                        "opacity": 1,
                                                                        "blendMode": "NORMAL",
                                                                        "color": {
                                                                            "r": 0.8196078538894653,
                                                                            "g": 0.8352941274642944,
                                                                            "b": 0.8627451062202454
                                                                        },
                                                                        "boundVariables": {}
                                                                    }
                                                                ],
                                                                "characters": "Discover raw, unfiltered music from indie artists breaking the mold. No corporate sellouts, just pure punk energy.",
                                                                "fontSize": 18,
                                                                "fontName": {
                                                                    "family": "Inter",
                                                                    "style": "Regular"
                                                                },
                                                                "textAlignHorizontal": "LEFT",
                                                                "textAlignVertical": "TOP",
                                                                "lineHeight": {
                                                                    "unit": "PIXELS",
                                                                    "value": 28
                                                                },
                                                                "tailwind": {
                                                                    "width": "w-561",
                                                                    "height": "h-56",
                                                                    "font-size": "text-18"
                                                                }
                                                            }
                                                        ],
                                                        "layoutMode": "NONE",
                                                        "primaryAxisSizingMode": "AUTO",
                                                        "counterAxisSizingMode": "FIXED",
                                                        "primaryAxisAlignItems": "MIN",
                                                        "counterAxisAlignItems": "MIN",
                                                        "tailwind": {
                                                            "width": "w-598",
                                                            "height": "h-56",
                                                            "margin-left": "ml-48",
                                                            "flex": "flex",
                                                            "justify-content": "justify-start",
                                                            "align-items": "items-start"
                                                        }
                                                    }
                                                ],
                                                "layoutMode": "VERTICAL",
                                                "primaryAxisSizingMode": "FIXED",
                                                "counterAxisSizingMode": "FIXED",
                                                "primaryAxisAlignItems": "MAX",
                                                "counterAxisAlignItems": "MIN",
                                                "paddingTop": 80,
                                                "paddingLeft": 48,
                                                "itemSpacing": 16,
                                                "tailwind": {
                                                    "width": "w-694",
                                                    "height": "h-320",
                                                    "padding": "pl-48 pt-80",
                                                    "flex": "flex flex-col gap-16",
                                                    "justify-content": "justify-end",
                                                    "align-items": "items-start"
                                                }
                                            }
                                        ],
                                        "layoutMode": "NONE",
                                        "primaryAxisSizingMode": "AUTO",
                                        "counterAxisSizingMode": "FIXED",
                                        "primaryAxisAlignItems": "MIN",
                                        "counterAxisAlignItems": "MIN",
                                        "tailwind": {
                                            "width": "w-694",
                                            "height": "h-320",
                                            "margin-left": "ml-32",
                                            "flex": "flex",
                                            "justify-content": "justify-start",
                                            "align-items": "items-start"
                                        }
                                    },
                                    {
                                        "name": "section",
                                        "type": "FRAME",
                                        "x": 32,
                                        "y": 400,
                                        "width": 694,
                                        "height": 208,
                                        "children": [
                                            {
                                                "name": "h2",
                                                "type": "FRAME",
                                                "x": 0,
                                                "y": 0,
                                                "width": 694,
                                                "height": 36,
                                                "children": [
                                                    {
                                                        "name": "CURATED CHAOS",
                                                        "type": "TEXT",
                                                        "x": 0,
                                                        "y": -2,
                                                        "width": 266,
                                                        "height": 36,
                                                        "fills": [
                                                            {
                                                                "type": "SOLID",
                                                                "visible": true,
                                                                "opacity": 1,
                                                                "blendMode": "NORMAL",
                                                                "color": {
                                                                    "r": 1,
                                                                    "g": 1,
                                                                    "b": 1
                                                                },
                                                                "boundVariables": {}
                                                            }
                                                        ],
                                                        "characters": "CURATED CHAOS",
                                                        "fontSize": 30,
                                                        "fontName": {
                                                            "family": "Inter",
                                                            "style": "Black"
                                                        },
                                                        "textAlignHorizontal": "LEFT",
                                                        "textAlignVertical": "TOP",
                                                        "lineHeight": {
                                                            "unit": "PIXELS",
                                                            "value": 36
                                                        },
                                                        "tailwind": {
                                                            "width": "w-266",
                                                            "height": "h-36",
                                                            "font-size": "text-30"
                                                        }
                                                    }
                                                ],
                                                "layoutMode": "NONE",
                                                "primaryAxisSizingMode": "AUTO",
                                                "counterAxisSizingMode": "FIXED",
                                                "primaryAxisAlignItems": "MIN",
                                                "counterAxisAlignItems": "MIN",
                                                "tailwind": {
                                                    "width": "w-694",
                                                    "height": "h-36",
                                                    "flex": "flex",
                                                    "justify-content": "justify-start",
                                                    "align-items": "items-start"
                                                }
                                            },
                                            {
                                                "name": "div",
                                                "type": "FRAME",
                                                "x": 0,
                                                "y": 60,
                                                "width": 694,
                                                "height": 148,
                                                "children": [
                                                    {
                                                        "name": "Container",
                                                        "type": "FRAME",
                                                        "x": 0,
                                                        "y": 0,
                                                        "width": 215.328125,
                                                        "height": 148,
                                                        "fills": [
                                                            {
                                                                "type": "SOLID",
                                                                "visible": true,
                                                                "opacity": 1,
                                                                "blendMode": "NORMAL",
                                                                "color": {
                                                                    "r": 0.0941176488995552,
                                                                    "g": 0.0941176488995552,
                                                                    "b": 0.10588235408067703
                                                                },
                                                                "boundVariables": {}
                                                            }
                                                        ],
                                                        "cornerRadius": 10,
                                                        "children": [
                                                            {
                                                                "name": "Container",
                                                                "type": "FRAME",
                                                                "x": 24,
                                                                "y": 24,
                                                                "width": 167.328125,
                                                                "height": 100,
                                                                "children": [
                                                                    {
                                                                        "type": "VECTOR",
                                                                        "x": 0,
                                                                        "y": 0,
                                                                        "width": 96,
                                                                        "height": 96
                                                                    },
                                                                    {
                                                                        "name": "Container",
                                                                        "type": "FRAME",
                                                                        "x": 112,
                                                                        "y": 0,
                                                                        "width": 55.328125,
                                                                        "height": 100,
                                                                        "children": [
                                                                            {
                                                                                "name": "h3",
                                                                                "type": "FRAME",
                                                                                "x": 0,
                                                                                "y": 0,
                                                                                "width": 55.328125,
                                                                                "height": 56,
                                                                                "children": [
                                                                                    {
                                                                                        "name": "Punk Essentials",
                                                                                        "type": "TEXT",
                                                                                        "x": 0,
                                                                                        "y": -1,
                                                                                        "width": 87,
                                                                                        "height": 84,
                                                                                        "fills": [
                                                                                            {
                                                                                                "type": "SOLID",
                                                                                                "visible": true,
                                                                                                "opacity": 1,
                                                                                                "blendMode": "NORMAL",
                                                                                                "color": {
                                                                                                    "r": 1,
                                                                                                    "g": 1,
                                                                                                    "b": 1
                                                                                                },
                                                                                                "boundVariables": {}
                                                                                            }
                                                                                        ],
                                                                                        "characters": "Punk Essentials",
                                                                                        "fontSize": 18,
                                                                                        "fontName": {
                                                                                            "family": "Inter",
                                                                                            "style": "Black"
                                                                                        },
                                                                                        "textAlignHorizontal": "LEFT",
                                                                                        "textAlignVertical": "TOP",
                                                                                        "lineHeight": {
                                                                                            "unit": "PIXELS",
                                                                                            "value": 28
                                                                                        },
                                                                                        "tailwind": {
                                                                                            "width": "w-87",
                                                                                            "height": "h-84",
                                                                                            "font-size": "text-18"
                                                                                        }
                                                                                    }
                                                                                ],
                                                                                "layoutMode": "NONE",
                                                                                "primaryAxisSizingMode": "AUTO",
                                                                                "counterAxisSizingMode": "FIXED",
                                                                                "primaryAxisAlignItems": "MIN",
                                                                                "counterAxisAlignItems": "MIN",
                                                                                "tailwind": {
                                                                                    "width": "w-55",
                                                                                    "height": "h-56",
                                                                                    "flex": "flex",
                                                                                    "justify-content": "justify-start",
                                                                                    "align-items": "items-start"
                                                                                }
                                                                            },
                                                                            {
                                                                                "name": "p",
                                                                                "type": "FRAME",
                                                                                "x": 0,
                                                                                "y": 60,
                                                                                "width": 55.328125,
                                                                                "height": 40,
                                                                                "children": [
                                                                                    {
                                                                                        "name": "The best of underground punk rock",
                                                                                        "type": "TEXT",
                                                                                        "x": 0,
                                                                                        "y": -0.5,
                                                                                        "width": 82,
                                                                                        "height": 80,
                                                                                        "fills": [
                                                                                            {
                                                                                                "type": "SOLID",
                                                                                                "visible": true,
                                                                                                "opacity": 1,
                                                                                                "blendMode": "NORMAL",
                                                                                                "color": {
                                                                                                    "r": 0.6000000238418579,
                                                                                                    "g": 0.6313725709915161,
                                                                                                    "b": 0.686274528503418
                                                                                                },
                                                                                                "boundVariables": {}
                                                                                            }
                                                                                        ],
                                                                                        "characters": "The best of underground punk rock",
                                                                                        "fontSize": 14,
                                                                                        "fontName": {
                                                                                            "family": "Inter",
                                                                                            "style": "Regular"
                                                                                        },
                                                                                        "textAlignHorizontal": "LEFT",
                                                                                        "textAlignVertical": "TOP",
                                                                                        "lineHeight": {
                                                                                            "unit": "PIXELS",
                                                                                            "value": 20
                                                                                        },
                                                                                        "tailwind": {
                                                                                            "width": "w-82",
                                                                                            "height": "h-80",
                                                                                            "font-size": "text-14"
                                                                                        }
                                                                                    }
                                                                                ],
                                                                                "layoutMode": "NONE",
                                                                                "primaryAxisSizingMode": "AUTO",
                                                                                "counterAxisSizingMode": "FIXED",
                                                                                "primaryAxisAlignItems": "MIN",
                                                                                "counterAxisAlignItems": "MIN",
                                                                                "tailwind": {
                                                                                    "width": "w-55",
                                                                                    "height": "h-40",
                                                                                    "flex": "flex",
                                                                                    "justify-content": "justify-start",
                                                                                    "align-items": "items-start"
                                                                                }
                                                                            }
                                                                        ],
                                                                        "layoutMode": "VERTICAL",
                                                                        "primaryAxisSizingMode": "FIXED",
                                                                        "counterAxisSizingMode": "FIXED",
                                                                        "primaryAxisAlignItems": "MIN",
                                                                        "counterAxisAlignItems": "MIN",
                                                                        "itemSpacing": 4,
                                                                        "tailwind": {
                                                                            "width": "w-55",
                                                                            "height": "h-100",
                                                                            "margin-left": "ml-112",
                                                                            "flex": "flex flex-col gap-4",
                                                                            "justify-content": "justify-start",
                                                                            "align-items": "items-start"
                                                                        }
                                                                    }
                                                                ],
                                                                "layoutMode": "HORIZONTAL",
                                                                "primaryAxisSizingMode": "FIXED",
                                                                "counterAxisSizingMode": "FIXED",
                                                                "primaryAxisAlignItems": "MIN",
                                                                "counterAxisAlignItems": "MIN",
                                                                "itemSpacing": 16,
                                                                "tailwind": {
                                                                    "width": "w-167",
                                                                    "height": "h-100",
                                                                    "margin-left": "ml-24",
                                                                    "flex": "flex gap-16",
                                                                    "justify-content": "justify-start",
                                                                    "align-items": "items-start"
                                                                }
                                                            }
                                                        ],
                                                        "layoutMode": "VERTICAL",
                                                        "primaryAxisSizingMode": "FIXED",
                                                        "counterAxisSizingMode": "FIXED",
                                                        "primaryAxisAlignItems": "MIN",
                                                        "counterAxisAlignItems": "MIN",
                                                        "paddingTop": 24,
                                                        "paddingRight": 24,
                                                        "paddingLeft": 24,
                                                        "tailwind": {
                                                            "width": "w-215",
                                                            "height": "h-148",
                                                            "padding": "px-24 pt-24",
                                                            "flex": "flex flex-col",
                                                            "justify-content": "justify-start",
                                                            "align-items": "items-start"
                                                        }
                                                    },
                                                    {
                                                        "name": "Container",
                                                        "type": "FRAME",
                                                        "x": 239.328125,
                                                        "y": 0,
                                                        "width": 215.3359375,
                                                        "height": 148,
                                                        "fills": [
                                                            {
                                                                "type": "SOLID",
                                                                "visible": true,
                                                                "opacity": 1,
                                                                "blendMode": "NORMAL",
                                                                "color": {
                                                                    "r": 0.0941176488995552,
                                                                    "g": 0.0941176488995552,
                                                                    "b": 0.10588235408067703
                                                                },
                                                                "boundVariables": {}
                                                            }
                                                        ],
                                                        "cornerRadius": 10,
                                                        "children": [
                                                            {
                                                                "name": "Container",
                                                                "type": "FRAME",
                                                                "x": 24,
                                                                "y": 24,
                                                                "width": 167.3359375,
                                                                "height": 100,
                                                                "children": [
                                                                    {
                                                                        "type": "VECTOR",
                                                                        "x": 0,
                                                                        "y": 0,
                                                                        "width": 96,
                                                                        "height": 96
                                                                    },
                                                                    {
                                                                        "name": "Container",
                                                                        "type": "FRAME",
                                                                        "x": 112,
                                                                        "y": 0,
                                                                        "width": 55.3359375,
                                                                        "height": 100,
                                                                        "children": [
                                                                            {
                                                                                "name": "h3",
                                                                                "type": "FRAME",
                                                                                "x": 0,
                                                                                "y": 0,
                                                                                "width": 55.3359375,
                                                                                "height": 56,
                                                                                "children": [
                                                                                    {
                                                                                        "name": "Indie Vibes",
                                                                                        "type": "TEXT",
                                                                                        "x": 0,
                                                                                        "y": -1,
                                                                                        "width": 49,
                                                                                        "height": 84,
                                                                                        "fills": [
                                                                                            {
                                                                                                "type": "SOLID",
                                                                                                "visible": true,
                                                                                                "opacity": 1,
                                                                                                "blendMode": "NORMAL",
                                                                                                "color": {
                                                                                                    "r": 1,
                                                                                                    "g": 1,
                                                                                                    "b": 1
                                                                                                },
                                                                                                "boundVariables": {}
                                                                                            }
                                                                                        ],
                                                                                        "characters": "Indie Vibes",
                                                                                        "fontSize": 18,
                                                                                        "fontName": {
                                                                                            "family": "Inter",
                                                                                            "style": "Black"
                                                                                        },
                                                                                        "textAlignHorizontal": "LEFT",
                                                                                        "textAlignVertical": "TOP",
                                                                                        "lineHeight": {
                                                                                            "unit": "PIXELS",
                                                                                            "value": 28
                                                                                        },
                                                                                        "tailwind": {
                                                                                            "width": "w-49",
                                                                                            "height": "h-84",
                                                                                            "font-size": "text-18"
                                                                                        }
                                                                                    }
                                                                                ],
                                                                                "layoutMode": "NONE",
                                                                                "primaryAxisSizingMode": "AUTO",
                                                                                "counterAxisSizingMode": "FIXED",
                                                                                "primaryAxisAlignItems": "MIN",
                                                                                "counterAxisAlignItems": "MIN",
                                                                                "tailwind": {
                                                                                    "width": "w-55",
                                                                                    "height": "h-56",
                                                                                    "flex": "flex",
                                                                                    "justify-content": "justify-start",
                                                                                    "align-items": "items-start"
                                                                                }
                                                                            },
                                                                            {
                                                                                "name": "p",
                                                                                "type": "FRAME",
                                                                                "x": 0,
                                                                                "y": 60,
                                                                                "width": 55.3359375,
                                                                                "height": 40,
                                                                                "children": [
                                                                                    {
                                                                                        "name": "Chill indie tracks for late nights",
                                                                                        "type": "TEXT",
                                                                                        "x": 0,
                                                                                        "y": -0.5,
                                                                                        "width": 44,
                                                                                        "height": 120,
                                                                                        "fills": [
                                                                                            {
                                                                                                "type": "SOLID",
                                                                                                "visible": true,
                                                                                                "opacity": 1,
                                                                                                "blendMode": "NORMAL",
                                                                                                "color": {
                                                                                                    "r": 0.6000000238418579,
                                                                                                    "g": 0.6313725709915161,
                                                                                                    "b": 0.686274528503418
                                                                                                },
                                                                                                "boundVariables": {}
                                                                                            }
                                                                                        ],
                                                                                        "characters": "Chill indie tracks for late nights",
                                                                                        "fontSize": 14,
                                                                                        "fontName": {
                                                                                            "family": "Inter",
                                                                                            "style": "Regular"
                                                                                        },
                                                                                        "textAlignHorizontal": "LEFT",
                                                                                        "textAlignVertical": "TOP",
                                                                                        "lineHeight": {
                                                                                            "unit": "PIXELS",
                                                                                            "value": 20
                                                                                        },
                                                                                        "tailwind": {
                                                                                            "width": "w-44",
                                                                                            "height": "h-120",
                                                                                            "font-size": "text-14"
                                                                                        }
                                                                                    }
                                                                                ],
                                                                                "layoutMode": "NONE",
                                                                                "primaryAxisSizingMode": "AUTO",
                                                                                "counterAxisSizingMode": "FIXED",
                                                                                "primaryAxisAlignItems": "MIN",
                                                                                "counterAxisAlignItems": "MIN",
                                                                                "tailwind": {
                                                                                    "width": "w-55",
                                                                                    "height": "h-40",
                                                                                    "flex": "flex",
                                                                                    "justify-content": "justify-start",
                                                                                    "align-items": "items-start"
                                                                                }
                                                                            }
                                                                        ],
                                                                        "layoutMode": "VERTICAL",
                                                                        "primaryAxisSizingMode": "FIXED",
                                                                        "counterAxisSizingMode": "FIXED",
                                                                        "primaryAxisAlignItems": "MIN",
                                                                        "counterAxisAlignItems": "MIN",
                                                                        "itemSpacing": 4,
                                                                        "tailwind": {
                                                                            "width": "w-55",
                                                                            "height": "h-100",
                                                                            "margin-left": "ml-112",
                                                                            "flex": "flex flex-col gap-4",
                                                                            "justify-content": "justify-start",
                                                                            "align-items": "items-start"
                                                                        }
                                                                    }
                                                                ],
                                                                "layoutMode": "HORIZONTAL",
                                                                "primaryAxisSizingMode": "FIXED",
                                                                "counterAxisSizingMode": "FIXED",
                                                                "primaryAxisAlignItems": "MIN",
                                                                "counterAxisAlignItems": "MIN",
                                                                "itemSpacing": 16,
                                                                "tailwind": {
                                                                    "width": "w-167",
                                                                    "height": "h-100",
                                                                    "margin-left": "ml-24",
                                                                    "flex": "flex gap-16",
                                                                    "justify-content": "justify-start",
                                                                    "align-items": "items-start"
                                                                }
                                                            }
                                                        ],
                                                        "layoutMode": "VERTICAL",
                                                        "primaryAxisSizingMode": "FIXED",
                                                        "counterAxisSizingMode": "FIXED",
                                                        "primaryAxisAlignItems": "MIN",
                                                        "counterAxisAlignItems": "MIN",
                                                        "paddingTop": 24,
                                                        "paddingRight": 24,
                                                        "paddingLeft": 24,
                                                        "tailwind": {
                                                            "width": "w-215",
                                                            "height": "h-148",
                                                            "margin-left": "ml-239",
                                                            "padding": "px-24 pt-24",
                                                            "flex": "flex flex-col",
                                                            "justify-content": "justify-start",
                                                            "align-items": "items-start"
                                                        }
                                                    },
                                                    {
                                                        "name": "Container",
                                                        "type": "FRAME",
                                                        "x": 478.6640625,
                                                        "y": 0,
                                                        "width": 215.328125,
                                                        "height": 148,
                                                        "fills": [
                                                            {
                                                                "type": "SOLID",
                                                                "visible": true,
                                                                "opacity": 1,
                                                                "blendMode": "NORMAL",
                                                                "color": {
                                                                    "r": 0.0941176488995552,
                                                                    "g": 0.0941176488995552,
                                                                    "b": 0.10588235408067703
                                                                },
                                                                "boundVariables": {}
                                                            }
                                                        ],
                                                        "cornerRadius": 10,
                                                        "children": [
                                                            {
                                                                "name": "Container",
                                                                "type": "FRAME",
                                                                "x": 24,
                                                                "y": 24,
                                                                "width": 167.328125,
                                                                "height": 100,
                                                                "children": [
                                                                    {
                                                                        "type": "VECTOR",
                                                                        "x": 0,
                                                                        "y": 0,
                                                                        "width": 96,
                                                                        "height": 96
                                                                    },
                                                                    {
                                                                        "name": "Container",
                                                                        "type": "FRAME",
                                                                        "x": 112,
                                                                        "y": 0,
                                                                        "width": 55.328125,
                                                                        "height": 100,
                                                                        "children": [
                                                                            {
                                                                                "name": "h3",
                                                                                "type": "FRAME",
                                                                                "x": 0,
                                                                                "y": 0,
                                                                                "width": 55.328125,
                                                                                "height": 56,
                                                                                "children": [
                                                                                    {
                                                                                        "name": "Raw Energy",
                                                                                        "type": "TEXT",
                                                                                        "x": 0,
                                                                                        "y": -1,
                                                                                        "width": 62,
                                                                                        "height": 84,
                                                                                        "fills": [
                                                                                            {
                                                                                                "type": "SOLID",
                                                                                                "visible": true,
                                                                                                "opacity": 1,
                                                                                                "blendMode": "NORMAL",
                                                                                                "color": {
                                                                                                    "r": 1,
                                                                                                    "g": 1,
                                                                                                    "b": 1
                                                                                                },
                                                                                                "boundVariables": {}
                                                                                            }
                                                                                        ],
                                                                                        "characters": "Raw Energy",
                                                                                        "fontSize": 18,
                                                                                        "fontName": {
                                                                                            "family": "Inter",
                                                                                            "style": "Black"
                                                                                        },
                                                                                        "textAlignHorizontal": "LEFT",
                                                                                        "textAlignVertical": "TOP",
                                                                                        "lineHeight": {
                                                                                            "unit": "PIXELS",
                                                                                            "value": 28
                                                                                        },
                                                                                        "tailwind": {
                                                                                            "width": "w-62",
                                                                                            "height": "h-84",
                                                                                            "font-size": "text-18"
                                                                                        }
                                                                                    }
                                                                                ],
                                                                                "layoutMode": "NONE",
                                                                                "primaryAxisSizingMode": "AUTO",
                                                                                "counterAxisSizingMode": "FIXED",
                                                                                "primaryAxisAlignItems": "MIN",
                                                                                "counterAxisAlignItems": "MIN",
                                                                                "tailwind": {
                                                                                    "width": "w-55",
                                                                                    "height": "h-56",
                                                                                    "flex": "flex",
                                                                                    "justify-content": "justify-start",
                                                                                    "align-items": "items-start"
                                                                                }
                                                                            },
                                                                            {
                                                                                "name": "p",
                                                                                "type": "FRAME",
                                                                                "x": 0,
                                                                                "y": 60,
                                                                                "width": 55.328125,
                                                                                "height": 40,
                                                                                "children": [
                                                                                    {
                                                                                        "name": "Fast, loud, and unfiltered",
                                                                                        "type": "TEXT",
                                                                                        "x": 0,
                                                                                        "y": -0.5,
                                                                                        "width": 60,
                                                                                        "height": 100,
                                                                                        "fills": [
                                                                                            {
                                                                                                "type": "SOLID",
                                                                                                "visible": true,
                                                                                                "opacity": 1,
                                                                                                "blendMode": "NORMAL",
                                                                                                "color": {
                                                                                                    "r": 0.6000000238418579,
                                                                                                    "g": 0.6313725709915161,
                                                                                                    "b": 0.686274528503418
                                                                                                },
                                                                                                "boundVariables": {}
                                                                                            }
                                                                                        ],
                                                                                        "characters": "Fast, loud, and unfiltered",
                                                                                        "fontSize": 14,
                                                                                        "fontName": {
                                                                                            "family": "Inter",
                                                                                            "style": "Regular"
                                                                                        },
                                                                                        "textAlignHorizontal": "LEFT",
                                                                                        "textAlignVertical": "TOP",
                                                                                        "lineHeight": {
                                                                                            "unit": "PIXELS",
                                                                                            "value": 20
                                                                                        },
                                                                                        "tailwind": {
                                                                                            "width": "w-60",
                                                                                            "height": "h-100",
                                                                                            "font-size": "text-14"
                                                                                        }
                                                                                    }
                                                                                ],
                                                                                "layoutMode": "NONE",
                                                                                "primaryAxisSizingMode": "AUTO",
                                                                                "counterAxisSizingMode": "FIXED",
                                                                                "primaryAxisAlignItems": "MIN",
                                                                                "counterAxisAlignItems": "MIN",
                                                                                "tailwind": {
                                                                                    "width": "w-55",
                                                                                    "height": "h-40",
                                                                                    "flex": "flex",
                                                                                    "justify-content": "justify-start",
                                                                                    "align-items": "items-start"
                                                                                }
                                                                            }
                                                                        ],
                                                                        "layoutMode": "VERTICAL",
                                                                        "primaryAxisSizingMode": "FIXED",
                                                                        "counterAxisSizingMode": "FIXED",
                                                                        "primaryAxisAlignItems": "MIN",
                                                                        "counterAxisAlignItems": "MIN",
                                                                        "itemSpacing": 4,
                                                                        "tailwind": {
                                                                            "width": "w-55",
                                                                            "height": "h-100",
                                                                            "margin-left": "ml-112",
                                                                            "flex": "flex flex-col gap-4",
                                                                            "justify-content": "justify-start",
                                                                            "align-items": "items-start"
                                                                        }
                                                                    }
                                                                ],
                                                                "layoutMode": "HORIZONTAL",
                                                                "primaryAxisSizingMode": "FIXED",
                                                                "counterAxisSizingMode": "FIXED",
                                                                "primaryAxisAlignItems": "MIN",
                                                                "counterAxisAlignItems": "MIN",
                                                                "itemSpacing": 16,
                                                                "tailwind": {
                                                                    "width": "w-167",
                                                                    "height": "h-100",
                                                                    "margin-left": "ml-24",
                                                                    "flex": "flex gap-16",
                                                                    "justify-content": "justify-start",
                                                                    "align-items": "items-start"
                                                                }
                                                            }
                                                        ],
                                                        "layoutMode": "VERTICAL",
                                                        "primaryAxisSizingMode": "FIXED",
                                                        "counterAxisSizingMode": "FIXED",
                                                        "primaryAxisAlignItems": "MIN",
                                                        "counterAxisAlignItems": "MIN",
                                                        "paddingTop": 24,
                                                        "paddingRight": 24,
                                                        "paddingLeft": 24,
                                                        "tailwind": {
                                                            "width": "w-215",
                                                            "height": "h-148",
                                                            "margin-left": "ml-479",
                                                            "padding": "px-24 pt-24",
                                                            "flex": "flex flex-col",
                                                            "justify-content": "justify-start",
                                                            "align-items": "items-start"
                                                        }
                                                    }
                                                ],
                                                "layoutMode": "NONE",
                                                "primaryAxisSizingMode": "AUTO",
                                                "counterAxisSizingMode": "FIXED",
                                                "primaryAxisAlignItems": "MIN",
                                                "counterAxisAlignItems": "MIN",
                                                "tailwind": {
                                                    "width": "w-694",
                                                    "height": "h-148",
                                                    "flex": "flex",
                                                    "justify-content": "justify-start",
                                                    "align-items": "items-start"
                                                }
                                            }
                                        ],
                                        "layoutMode": "VERTICAL",
                                        "primaryAxisSizingMode": "FIXED",
                                        "counterAxisSizingMode": "FIXED",
                                        "primaryAxisAlignItems": "MIN",
                                        "counterAxisAlignItems": "MIN",
                                        "itemSpacing": 24,
                                        "tailwind": {
                                            "width": "w-694",
                                            "height": "h-208",
                                            "margin-left": "ml-32",
                                            "flex": "flex flex-col gap-24",
                                            "justify-content": "justify-start",
                                            "align-items": "items-start"
                                        }
                                    },
                                    {
                                        "name": "section",
                                        "type": "FRAME",
                                        "x": 32,
                                        "y": 656,
                                        "width": 694,
                                        "height": 378.3359375,
                                        "children": [
                                            {
                                                "name": "div",
                                                "type": "FRAME",
                                                "x": 0,
                                                "y": 0,
                                                "width": 694,
                                                "height": 36,
                                                "children": [
                                                    {
                                                        "name": "h2",
                                                        "type": "FRAME",
                                                        "x": 0,
                                                        "y": 0,
                                                        "width": 272.375,
                                                        "height": 36,
                                                        "children": [
                                                            {
                                                                "name": "FEATURED REBELS",
                                                                "type": "TEXT",
                                                                "x": 0,
                                                                "y": -2,
                                                                "width": 281,
                                                                "height": 36,
                                                                "fills": [
                                                                    {
                                                                        "type": "SOLID",
                                                                        "visible": true,
                                                                        "opacity": 1,
                                                                        "blendMode": "NORMAL",
                                                                        "color": {
                                                                            "r": 1,
                                                                            "g": 1,
                                                                            "b": 1
                                                                        },
                                                                        "boundVariables": {}
                                                                    }
                                                                ],
                                                                "characters": "FEATURED REBELS",
                                                                "fontSize": 30,
                                                                "fontName": {
                                                                    "family": "Inter",
                                                                    "style": "Black"
                                                                },
                                                                "textAlignHorizontal": "LEFT",
                                                                "textAlignVertical": "TOP",
                                                                "lineHeight": {
                                                                    "unit": "PIXELS",
                                                                    "value": 36
                                                                },
                                                                "tailwind": {
                                                                    "width": "w-281",
                                                                    "height": "h-36",
                                                                    "font-size": "text-30"
                                                                }
                                                            }
                                                        ],
                                                        "layoutMode": "NONE",
                                                        "primaryAxisSizingMode": "AUTO",
                                                        "counterAxisSizingMode": "FIXED",
                                                        "primaryAxisAlignItems": "MIN",
                                                        "counterAxisAlignItems": "MIN",
                                                        "tailwind": {
                                                            "width": "w-272",
                                                            "height": "h-36",
                                                            "flex": "flex",
                                                            "justify-content": "justify-start",
                                                            "align-items": "items-start"
                                                        }
                                                    },
                                                    {
                                                        "name": "Link",
                                                        "type": "FRAME",
                                                        "x": 617.25,
                                                        "y": 6,
                                                        "width": 76.75,
                                                        "height": 24,
                                                        "children": [
                                                            {
                                                                "name": "View All →",
                                                                "type": "TEXT",
                                                                "x": 0,
                                                                "y": -2,
                                                                "width": 82,
                                                                "height": 24,
                                                                "fills": [
                                                                    {
                                                                        "type": "SOLID",
                                                                        "visible": true,
                                                                        "opacity": 1,
                                                                        "blendMode": "NORMAL",
                                                                        "color": {
                                                                            "r": 0.9647058844566345,
                                                                            "g": 0.20000000298023224,
                                                                            "b": 0.6039215922355652
                                                                        },
                                                                        "boundVariables": {}
                                                                    }
                                                                ],
                                                                "characters": "View All →",
                                                                "fontSize": 16,
                                                                "fontName": {
                                                                    "family": "Inter",
                                                                    "style": "Semi Bold"
                                                                },
                                                                "textAlignHorizontal": "LEFT",
                                                                "textAlignVertical": "TOP",
                                                                "lineHeight": {
                                                                    "unit": "PIXELS",
                                                                    "value": 24
                                                                },
                                                                "tailwind": {
                                                                    "width": "w-82",
                                                                    "height": "h-24",
                                                                    "font-size": "text-16"
                                                                }
                                                            }
                                                        ],
                                                        "layoutMode": "NONE",
                                                        "primaryAxisSizingMode": "AUTO",
                                                        "counterAxisSizingMode": "FIXED",
                                                        "primaryAxisAlignItems": "MIN",
                                                        "counterAxisAlignItems": "MIN",
                                                        "tailwind": {
                                                            "width": "w-77",
                                                            "height": "h-24",
                                                            "margin-left": "ml-617",
                                                            "flex": "flex",
                                                            "justify-content": "justify-start",
                                                            "align-items": "items-start"
                                                        }
                                                    }
                                                ],
                                                "layoutMode": "HORIZONTAL",
                                                "primaryAxisSizingMode": "FIXED",
                                                "counterAxisSizingMode": "FIXED",
                                                "primaryAxisAlignItems": "SPACE_BETWEEN",
                                                "counterAxisAlignItems": "CENTER",
                                                "tailwind": {
                                                    "width": "w-694",
                                                    "height": "h-36",
                                                    "flex": "flex",
                                                    "justify-content": "justify-between",
                                                    "align-items": "items-center"
                                                }
                                            },
                                            {
                                                "name": "div",
                                                "type": "FRAME",
                                                "x": 0,
                                                "y": 60,
                                                "width": 694,
                                                "height": 318.3359375,
                                                "children": [
                                                    {
                                                        "name": "Link",
                                                        "type": "FRAME",
                                                        "x": 0,
                                                        "y": 0,
                                                        "width": 215.328125,
                                                        "height": 318.3359375,
                                                        "fills": [
                                                            {
                                                                "type": "SOLID",
                                                                "visible": true,
                                                                "opacity": 1,
                                                                "blendMode": "NORMAL",
                                                                "color": {
                                                                    "r": 0.0941176488995552,
                                                                    "g": 0.0941176488995552,
                                                                    "b": 0.10588235408067703
                                                                },
                                                                "boundVariables": {}
                                                            }
                                                        ],
                                                        "cornerRadius": 10,
                                                        "children": [
                                                            {
                                                                "name": "div",
                                                                "type": "FRAME",
                                                                "x": 0,
                                                                "y": 0,
                                                                "width": 215.328125,
                                                                "height": 215.328125,
                                                                "children": [
                                                                    {
                                                                        "type": "VECTOR",
                                                                        "x": 0,
                                                                        "y": 0,
                                                                        "width": 215.328125,
                                                                        "height": 215.328125
                                                                    },
                                                                    {
                                                                        "type": "VECTOR",
                                                                        "x": 0,
                                                                        "y": 0,
                                                                        "width": 215.328125,
                                                                        "height": 215.328125
                                                                    },
                                                                    {
                                                                        "name": "Container",
                                                                        "type": "FRAME",
                                                                        "x": 129.9453125,
                                                                        "y": 8,
                                                                        "width": 77.3828125,
                                                                        "height": 26,
                                                                        "fills": [
                                                                            {
                                                                                "type": "SOLID",
                                                                                "visible": true,
                                                                                "opacity": 0.6000000238418579,
                                                                                "blendMode": "NORMAL",
                                                                                "color": {
                                                                                    "r": 0,
                                                                                    "g": 0,
                                                                                    "b": 0
                                                                                },
                                                                                "boundVariables": {}
                                                                            }
                                                                        ],
                                                                        "strokes": [
                                                                            {
                                                                                "type": "SOLID",
                                                                                "opacity": 0.30000001192092896,
                                                                                "blendMode": "NORMAL",
                                                                                "color": "#00d3f3"
                                                                            }
                                                                        ],
                                                                        "cornerRadius": 4,
                                                                        "children": [
                                                                            {
                                                                                "name": "Punk Rock",
                                                                                "type": "TEXT",
                                                                                "x": 9,
                                                                                "y": 5,
                                                                                "width": 60,
                                                                                "height": 16,
                                                                                "fills": [
                                                                                    {
                                                                                        "type": "SOLID",
                                                                                        "visible": true,
                                                                                        "opacity": 1,
                                                                                        "blendMode": "NORMAL",
                                                                                        "color": {
                                                                                            "r": 0,
                                                                                            "g": 0.8274509906768799,
                                                                                            "b": 0.9529411792755127
                                                                                        },
                                                                                        "boundVariables": {}
                                                                                    }
                                                                                ],
                                                                                "characters": "Punk Rock",
                                                                                "fontSize": 12,
                                                                                "fontName": {
                                                                                    "family": "Consolas",
                                                                                    "style": "Regular"
                                                                                },
                                                                                "textAlignHorizontal": "LEFT",
                                                                                "textAlignVertical": "TOP",
                                                                                "lineHeight": {
                                                                                    "unit": "PIXELS",
                                                                                    "value": 16
                                                                                },
                                                                                "tailwind": {
                                                                                    "width": "w-60",
                                                                                    "height": "h-16",
                                                                                    "margin-left": "ml-9",
                                                                                    "font-size": "text-12"
                                                                                }
                                                                            }
                                                                        ],
                                                                        "layoutMode": "NONE",
                                                                        "primaryAxisSizingMode": "AUTO",
                                                                        "counterAxisSizingMode": "FIXED",
                                                                        "primaryAxisAlignItems": "MIN",
                                                                        "counterAxisAlignItems": "MIN",
                                                                        "paddingTop": 1,
                                                                        "paddingRight": 1,
                                                                        "paddingBottom": 1,
                                                                        "paddingLeft": 1,
                                                                        "tailwind": {
                                                                            "width": "w-77",
                                                                            "height": "h-26",
                                                                            "margin-left": "ml-130",
                                                                            "padding": "p-1",
                                                                            "flex": "flex",
                                                                            "justify-content": "justify-start",
                                                                            "align-items": "items-start"
                                                                        }
                                                                    }
                                                                ],
                                                                "layoutMode": "NONE",
                                                                "primaryAxisSizingMode": "AUTO",
                                                                "counterAxisSizingMode": "FIXED",
                                                                "primaryAxisAlignItems": "MIN",
                                                                "counterAxisAlignItems": "MIN",
                                                                "tailwind": {
                                                                    "width": "w-215",
                                                                    "height": "h-215",
                                                                    "flex": "flex",
                                                                    "justify-content": "justify-start",
                                                                    "align-items": "items-start"
                                                                }
                                                            },
                                                            {
                                                                "name": "div",
                                                                "type": "FRAME",
                                                                "x": 0,
                                                                "y": 215.328125,
                                                                "width": 215.328125,
                                                                "height": 103,
                                                                "children": [
                                                                    {
                                                                        "name": "h3",
                                                                        "type": "FRAME",
                                                                        "x": 16,
                                                                        "y": 16,
                                                                        "width": 183.328125,
                                                                        "height": 27,
                                                                        "children": [
                                                                            {
                                                                                "name": "The Reckless Hearts",
                                                                                "type": "TEXT",
                                                                                "x": 0,
                                                                                "y": -0.5,
                                                                                "width": 186,
                                                                                "height": 27,
                                                                                "fills": [
                                                                                    {
                                                                                        "type": "SOLID",
                                                                                        "visible": true,
                                                                                        "opacity": 1,
                                                                                        "blendMode": "NORMAL",
                                                                                        "color": {
                                                                                            "r": 1,
                                                                                            "g": 1,
                                                                                            "b": 1
                                                                                        },
                                                                                        "boundVariables": {}
                                                                                    }
                                                                                ],
                                                                                "characters": "The Reckless Hearts",
                                                                                "fontSize": 18,
                                                                                "fontName": {
                                                                                    "family": "Inter",
                                                                                    "style": "Black"
                                                                                },
                                                                                "textAlignHorizontal": "LEFT",
                                                                                "textAlignVertical": "TOP",
                                                                                "lineHeight": {
                                                                                    "unit": "PIXELS",
                                                                                    "value": 27
                                                                                },
                                                                                "tailwind": {
                                                                                    "width": "w-186",
                                                                                    "height": "h-27",
                                                                                    "font-size": "text-18"
                                                                                }
                                                                            }
                                                                        ],
                                                                        "layoutMode": "NONE",
                                                                        "primaryAxisSizingMode": "AUTO",
                                                                        "counterAxisSizingMode": "FIXED",
                                                                        "primaryAxisAlignItems": "MIN",
                                                                        "counterAxisAlignItems": "MIN",
                                                                        "tailwind": {
                                                                            "width": "w-183",
                                                                            "height": "h-27",
                                                                            "margin-left": "ml-16",
                                                                            "flex": "flex",
                                                                            "justify-content": "justify-start",
                                                                            "align-items": "items-start"
                                                                        }
                                                                    },
                                                                    {
                                                                        "name": "p",
                                                                        "type": "FRAME",
                                                                        "x": 16,
                                                                        "y": 47,
                                                                        "width": 183.328125,
                                                                        "height": 40,
                                                                        "children": [
                                                                            {
                                                                                "name": "Raw energy meets melodic chaos. DIY punk from the ",
                                                                                "type": "TEXT",
                                                                                "x": 0,
                                                                                "y": -0.5,
                                                                                "width": 167,
                                                                                "height": 60,
                                                                                "fills": [
                                                                                    {
                                                                                        "type": "SOLID",
                                                                                        "visible": true,
                                                                                        "opacity": 1,
                                                                                        "blendMode": "NORMAL",
                                                                                        "color": {
                                                                                            "r": 0.4156862795352936,
                                                                                            "g": 0.4470588266849518,
                                                                                            "b": 0.5098039507865906
                                                                                        },
                                                                                        "boundVariables": {}
                                                                                    }
                                                                                ],
                                                                                "characters": "Raw energy meets melodic chaos. DIY punk from the underground.",
                                                                                "fontSize": 14,
                                                                                "fontName": {
                                                                                    "family": "Inter",
                                                                                    "style": "Regular"
                                                                                },
                                                                                "textAlignHorizontal": "LEFT",
                                                                                "textAlignVertical": "TOP",
                                                                                "lineHeight": {
                                                                                    "unit": "PIXELS",
                                                                                    "value": 20
                                                                                },
                                                                                "tailwind": {
                                                                                    "width": "w-167",
                                                                                    "height": "h-60",
                                                                                    "font-size": "text-14"
                                                                                }
                                                                            }
                                                                        ],
                                                                        "layoutMode": "NONE",
                                                                        "primaryAxisSizingMode": "AUTO",
                                                                        "counterAxisSizingMode": "FIXED",
                                                                        "primaryAxisAlignItems": "MIN",
                                                                        "counterAxisAlignItems": "MIN",
                                                                        "tailwind": {
                                                                            "width": "w-183",
                                                                            "height": "h-40",
                                                                            "margin-left": "ml-16",
                                                                            "flex": "flex",
                                                                            "justify-content": "justify-start",
                                                                            "align-items": "items-start"
                                                                        }
                                                                    }
                                                                ],
                                                                "layoutMode": "VERTICAL",
                                                                "primaryAxisSizingMode": "FIXED",
                                                                "counterAxisSizingMode": "FIXED",
                                                                "primaryAxisAlignItems": "MIN",
                                                                "counterAxisAlignItems": "MIN",
                                                                "paddingTop": 16,
                                                                "paddingRight": 16,
                                                                "paddingLeft": 16,
                                                                "itemSpacing": 4,
                                                                "tailwind": {
                                                                    "width": "w-215",
                                                                    "height": "h-103",
                                                                    "padding": "px-16 pt-16",
                                                                    "flex": "flex flex-col gap-4",
                                                                    "justify-content": "justify-start",
                                                                    "align-items": "items-start"
                                                                }
                                                            }
                                                        ],
                                                        "layoutMode": "VERTICAL",
                                                        "primaryAxisSizingMode": "FIXED",
                                                        "counterAxisSizingMode": "FIXED",
                                                        "primaryAxisAlignItems": "MIN",
                                                        "counterAxisAlignItems": "MIN",
                                                        "tailwind": {
                                                            "width": "w-215",
                                                            "height": "h-318",
                                                            "flex": "flex flex-col",
                                                            "justify-content": "justify-start",
                                                            "align-items": "items-start"
                                                        }
                                                    },
                                                    {
                                                        "name": "Link",
                                                        "type": "FRAME",
                                                        "x": 239.328125,
                                                        "y": 0,
                                                        "width": 215.3359375,
                                                        "height": 318.3359375,
                                                        "fills": [
                                                            {
                                                                "type": "SOLID",
                                                                "visible": true,
                                                                "opacity": 1,
                                                                "blendMode": "NORMAL",
                                                                "color": {
                                                                    "r": 0.0941176488995552,
                                                                    "g": 0.0941176488995552,
                                                                    "b": 0.10588235408067703
                                                                },
                                                                "boundVariables": {}
                                                            }
                                                        ],
                                                        "cornerRadius": 10,
                                                        "children": [
                                                            {
                                                                "name": "div",
                                                                "type": "FRAME",
                                                                "x": 0,
                                                                "y": 0,
                                                                "width": 215.3359375,
                                                                "height": 215.3359375,
                                                                "children": [
                                                                    {
                                                                        "type": "VECTOR",
                                                                        "x": 0,
                                                                        "y": 0,
                                                                        "width": 215.3359375,
                                                                        "height": 215.3359375
                                                                    },
                                                                    {
                                                                        "type": "VECTOR",
                                                                        "x": 0,
                                                                        "y": 0,
                                                                        "width": 215.3359375,
                                                                        "height": 215.3359375
                                                                    },
                                                                    {
                                                                        "name": "Container",
                                                                        "type": "FRAME",
                                                                        "x": 123.359375,
                                                                        "y": 8,
                                                                        "width": 83.9765625,
                                                                        "height": 26,
                                                                        "fills": [
                                                                            {
                                                                                "type": "SOLID",
                                                                                "visible": true,
                                                                                "opacity": 0.6000000238418579,
                                                                                "blendMode": "NORMAL",
                                                                                "color": {
                                                                                    "r": 0,
                                                                                    "g": 0,
                                                                                    "b": 0
                                                                                },
                                                                                "boundVariables": {}
                                                                            }
                                                                        ],
                                                                        "strokes": [
                                                                            {
                                                                                "type": "SOLID",
                                                                                "opacity": 0.30000001192092896,
                                                                                "blendMode": "NORMAL",
                                                                                "color": "#00d3f3"
                                                                            }
                                                                        ],
                                                                        "cornerRadius": 4,
                                                                        "children": [
                                                                            {
                                                                                "name": "Indie Rock",
                                                                                "type": "TEXT",
                                                                                "x": 9,
                                                                                "y": 5,
                                                                                "width": 66,
                                                                                "height": 16,
                                                                                "fills": [
                                                                                    {
                                                                                        "type": "SOLID",
                                                                                        "visible": true,
                                                                                        "opacity": 1,
                                                                                        "blendMode": "NORMAL",
                                                                                        "color": {
                                                                                            "r": 0,
                                                                                            "g": 0.8274509906768799,
                                                                                            "b": 0.9529411792755127
                                                                                        },
                                                                                        "boundVariables": {}
                                                                                    }
                                                                                ],
                                                                                "characters": "Indie Rock",
                                                                                "fontSize": 12,
                                                                                "fontName": {
                                                                                    "family": "Consolas",
                                                                                    "style": "Regular"
                                                                                },
                                                                                "textAlignHorizontal": "LEFT",
                                                                                "textAlignVertical": "TOP",
                                                                                "lineHeight": {
                                                                                    "unit": "PIXELS",
                                                                                    "value": 16
                                                                                },
                                                                                "tailwind": {
                                                                                    "width": "w-66",
                                                                                    "height": "h-16",
                                                                                    "margin-left": "ml-9",
                                                                                    "font-size": "text-12"
                                                                                }
                                                                            }
                                                                        ],
                                                                        "layoutMode": "NONE",
                                                                        "primaryAxisSizingMode": "AUTO",
                                                                        "counterAxisSizingMode": "FIXED",
                                                                        "primaryAxisAlignItems": "MIN",
                                                                        "counterAxisAlignItems": "MIN",
                                                                        "paddingTop": 1,
                                                                        "paddingRight": 1,
                                                                        "paddingBottom": 1,
                                                                        "paddingLeft": 1,
                                                                        "tailwind": {
                                                                            "width": "w-84",
                                                                            "height": "h-26",
                                                                            "margin-left": "ml-123",
                                                                            "padding": "p-1",
                                                                            "flex": "flex",
                                                                            "justify-content": "justify-start",
                                                                            "align-items": "items-start"
                                                                        }
                                                                    }
                                                                ],
                                                                "layoutMode": "NONE",
                                                                "primaryAxisSizingMode": "AUTO",
                                                                "counterAxisSizingMode": "FIXED",
                                                                "primaryAxisAlignItems": "MIN",
                                                                "counterAxisAlignItems": "MIN",
                                                                "tailwind": {
                                                                    "width": "w-215",
                                                                    "height": "h-215",
                                                                    "flex": "flex",
                                                                    "justify-content": "justify-start",
                                                                    "align-items": "items-start"
                                                                }
                                                            },
                                                            {
                                                                "name": "div",
                                                                "type": "FRAME",
                                                                "x": 0,
                                                                "y": 215.3359375,
                                                                "width": 215.3359375,
                                                                "height": 103,
                                                                "children": [
                                                                    {
                                                                        "name": "h3",
                                                                        "type": "FRAME",
                                                                        "x": 16,
                                                                        "y": 16,
                                                                        "width": 183.3359375,
                                                                        "height": 27,
                                                                        "children": [
                                                                            {
                                                                                "name": "Luna Crash",
                                                                                "type": "TEXT",
                                                                                "x": 0,
                                                                                "y": -0.5,
                                                                                "width": 102,
                                                                                "height": 27,
                                                                                "fills": [
                                                                                    {
                                                                                        "type": "SOLID",
                                                                                        "visible": true,
                                                                                        "opacity": 1,
                                                                                        "blendMode": "NORMAL",
                                                                                        "color": {
                                                                                            "r": 1,
                                                                                            "g": 1,
                                                                                            "b": 1
                                                                                        },
                                                                                        "boundVariables": {}
                                                                                    }
                                                                                ],
                                                                                "characters": "Luna Crash",
                                                                                "fontSize": 18,
                                                                                "fontName": {
                                                                                    "family": "Inter",
                                                                                    "style": "Black"
                                                                                },
                                                                                "textAlignHorizontal": "LEFT",
                                                                                "textAlignVertical": "TOP",
                                                                                "lineHeight": {
                                                                                    "unit": "PIXELS",
                                                                                    "value": 27
                                                                                },
                                                                                "tailwind": {
                                                                                    "width": "w-102",
                                                                                    "height": "h-27",
                                                                                    "font-size": "text-18"
                                                                                }
                                                                            }
                                                                        ],
                                                                        "layoutMode": "NONE",
                                                                        "primaryAxisSizingMode": "AUTO",
                                                                        "counterAxisSizingMode": "FIXED",
                                                                        "primaryAxisAlignItems": "MIN",
                                                                        "counterAxisAlignItems": "MIN",
                                                                        "tailwind": {
                                                                            "width": "w-183",
                                                                            "height": "h-27",
                                                                            "margin-left": "ml-16",
                                                                            "flex": "flex",
                                                                            "justify-content": "justify-start",
                                                                            "align-items": "items-start"
                                                                        }
                                                                    },
                                                                    {
                                                                        "name": "p",
                                                                        "type": "FRAME",
                                                                        "x": 16,
                                                                        "y": 47,
                                                                        "width": 183.3359375,
                                                                        "height": 40,
                                                                        "children": [
                                                                            {
                                                                                "name": "Ethereal vocals meet distorted guitars. Indie rock",
                                                                                "type": "TEXT",
                                                                                "x": 0,
                                                                                "y": -0.5,
                                                                                "width": 170,
                                                                                "height": 60,
                                                                                "fills": [
                                                                                    {
                                                                                        "type": "SOLID",
                                                                                        "visible": true,
                                                                                        "opacity": 1,
                                                                                        "blendMode": "NORMAL",
                                                                                        "color": {
                                                                                            "r": 0.4156862795352936,
                                                                                            "g": 0.4470588266849518,
                                                                                            "b": 0.5098039507865906
                                                                                        },
                                                                                        "boundVariables": {}
                                                                                    }
                                                                                ],
                                                                                "characters": "Ethereal vocals meet distorted guitars. Indie rock with an edge.",
                                                                                "fontSize": 14,
                                                                                "fontName": {
                                                                                    "family": "Inter",
                                                                                    "style": "Regular"
                                                                                },
                                                                                "textAlignHorizontal": "LEFT",
                                                                                "textAlignVertical": "TOP",
                                                                                "lineHeight": {
                                                                                    "unit": "PIXELS",
                                                                                    "value": 20
                                                                                },
                                                                                "tailwind": {
                                                                                    "width": "w-170",
                                                                                    "height": "h-60",
                                                                                    "font-size": "text-14"
                                                                                }
                                                                            }
                                                                        ],
                                                                        "layoutMode": "NONE",
                                                                        "primaryAxisSizingMode": "AUTO",
                                                                        "counterAxisSizingMode": "FIXED",
                                                                        "primaryAxisAlignItems": "MIN",
                                                                        "counterAxisAlignItems": "MIN",
                                                                        "tailwind": {
                                                                            "width": "w-183",
                                                                            "height": "h-40",
                                                                            "margin-left": "ml-16",
                                                                            "flex": "flex",
                                                                            "justify-content": "justify-start",
                                                                            "align-items": "items-start"
                                                                        }
                                                                    }
                                                                ],
                                                                "layoutMode": "VERTICAL",
                                                                "primaryAxisSizingMode": "FIXED",
                                                                "counterAxisSizingMode": "FIXED",
                                                                "primaryAxisAlignItems": "MIN",
                                                                "counterAxisAlignItems": "MIN",
                                                                "paddingTop": 16,
                                                                "paddingRight": 16,
                                                                "paddingLeft": 16,
                                                                "itemSpacing": 4,
                                                                "tailwind": {
                                                                    "width": "w-215",
                                                                    "height": "h-103",
                                                                    "padding": "px-16 pt-16",
                                                                    "flex": "flex flex-col gap-4",
                                                                    "justify-content": "justify-start",
                                                                    "align-items": "items-start"
                                                                }
                                                            }
                                                        ],
                                                        "layoutMode": "VERTICAL",
                                                        "primaryAxisSizingMode": "FIXED",
                                                        "counterAxisSizingMode": "FIXED",
                                                        "primaryAxisAlignItems": "MIN",
                                                        "counterAxisAlignItems": "MIN",
                                                        "tailwind": {
                                                            "width": "w-215",
                                                            "height": "h-318",
                                                            "margin-left": "ml-239",
                                                            "flex": "flex flex-col",
                                                            "justify-content": "justify-start",
                                                            "align-items": "items-start"
                                                        }
                                                    },
                                                    {
                                                        "name": "Link",
                                                        "type": "FRAME",
                                                        "x": 478.6640625,
                                                        "y": 0,
                                                        "width": 215.328125,
                                                        "height": 318.3359375,
                                                        "fills": [
                                                            {
                                                                "type": "SOLID",
                                                                "visible": true,
                                                                "opacity": 1,
                                                                "blendMode": "NORMAL",
                                                                "color": {
                                                                    "r": 0.0941176488995552,
                                                                    "g": 0.0941176488995552,
                                                                    "b": 0.10588235408067703
                                                                },
                                                                "boundVariables": {}
                                                            }
                                                        ],
                                                        "cornerRadius": 10,
                                                        "children": [
                                                            {
                                                                "name": "div",
                                                                "type": "FRAME",
                                                                "x": 0,
                                                                "y": 0,
                                                                "width": 215.328125,
                                                                "height": 215.328125,
                                                                "children": [
                                                                    {
                                                                        "type": "VECTOR",
                                                                        "x": 0,
                                                                        "y": 0,
                                                                        "width": 215.328125,
                                                                        "height": 215.328125
                                                                    },
                                                                    {
                                                                        "type": "VECTOR",
                                                                        "x": 0,
                                                                        "y": 0,
                                                                        "width": 215.328125,
                                                                        "height": 215.328125
                                                                    },
                                                                    {
                                                                        "name": "Container",
                                                                        "type": "FRAME",
                                                                        "x": 116.75,
                                                                        "y": 8,
                                                                        "width": 90.578125,
                                                                        "height": 26,
                                                                        "fills": [
                                                                            {
                                                                                "type": "SOLID",
                                                                                "visible": true,
                                                                                "opacity": 0.6000000238418579,
                                                                                "blendMode": "NORMAL",
                                                                                "color": {
                                                                                    "r": 0,
                                                                                    "g": 0,
                                                                                    "b": 0
                                                                                },
                                                                                "boundVariables": {}
                                                                            }
                                                                        ],
                                                                        "strokes": [
                                                                            {
                                                                                "type": "SOLID",
                                                                                "opacity": 0.30000001192092896,
                                                                                "blendMode": "NORMAL",
                                                                                "color": "#00d3f3"
                                                                            }
                                                                        ],
                                                                        "cornerRadius": 4,
                                                                        "children": [
                                                                            {
                                                                                "name": "Alternative",
                                                                                "type": "TEXT",
                                                                                "x": 9,
                                                                                "y": 5,
                                                                                "width": 73,
                                                                                "height": 16,
                                                                                "fills": [
                                                                                    {
                                                                                        "type": "SOLID",
                                                                                        "visible": true,
                                                                                        "opacity": 1,
                                                                                        "blendMode": "NORMAL",
                                                                                        "color": {
                                                                                            "r": 0,
                                                                                            "g": 0.8274509906768799,
                                                                                            "b": 0.9529411792755127
                                                                                        },
                                                                                        "boundVariables": {}
                                                                                    }
                                                                                ],
                                                                                "characters": "Alternative",
                                                                                "fontSize": 12,
                                                                                "fontName": {
                                                                                    "family": "Consolas",
                                                                                    "style": "Regular"
                                                                                },
                                                                                "textAlignHorizontal": "LEFT",
                                                                                "textAlignVertical": "TOP",
                                                                                "lineHeight": {
                                                                                    "unit": "PIXELS",
                                                                                    "value": 16
                                                                                },
                                                                                "tailwind": {
                                                                                    "width": "w-73",
                                                                                    "height": "h-16",
                                                                                    "margin-left": "ml-9",
                                                                                    "font-size": "text-12"
                                                                                }
                                                                            }
                                                                        ],
                                                                        "layoutMode": "NONE",
                                                                        "primaryAxisSizingMode": "AUTO",
                                                                        "counterAxisSizingMode": "FIXED",
                                                                        "primaryAxisAlignItems": "MIN",
                                                                        "counterAxisAlignItems": "MIN",
                                                                        "paddingTop": 1,
                                                                        "paddingRight": 1,
                                                                        "paddingBottom": 1,
                                                                        "paddingLeft": 1,
                                                                        "tailwind": {
                                                                            "width": "w-91",
                                                                            "height": "h-26",
                                                                            "margin-left": "ml-117",
                                                                            "padding": "p-1",
                                                                            "flex": "flex",
                                                                            "justify-content": "justify-start",
                                                                            "align-items": "items-start"
                                                                        }
                                                                    }
                                                                ],
                                                                "layoutMode": "NONE",
                                                                "primaryAxisSizingMode": "AUTO",
                                                                "counterAxisSizingMode": "FIXED",
                                                                "primaryAxisAlignItems": "MIN",
                                                                "counterAxisAlignItems": "MIN",
                                                                "tailwind": {
                                                                    "width": "w-215",
                                                                    "height": "h-215",
                                                                    "flex": "flex",
                                                                    "justify-content": "justify-start",
                                                                    "align-items": "items-start"
                                                                }
                                                            },
                                                            {
                                                                "name": "div",
                                                                "type": "FRAME",
                                                                "x": 0,
                                                                "y": 215.328125,
                                                                "width": 215.328125,
                                                                "height": 103,
                                                                "children": [
                                                                    {
                                                                        "name": "h3",
                                                                        "type": "FRAME",
                                                                        "x": 16,
                                                                        "y": 16,
                                                                        "width": 183.328125,
                                                                        "height": 27,
                                                                        "children": [
                                                                            {
                                                                                "name": "Neon Wasteland",
                                                                                "type": "TEXT",
                                                                                "x": 0,
                                                                                "y": -0.5,
                                                                                "width": 148,
                                                                                "height": 27,
                                                                                "fills": [
                                                                                    {
                                                                                        "type": "SOLID",
                                                                                        "visible": true,
                                                                                        "opacity": 1,
                                                                                        "blendMode": "NORMAL",
                                                                                        "color": {
                                                                                            "r": 1,
                                                                                            "g": 1,
                                                                                            "b": 1
                                                                                        },
                                                                                        "boundVariables": {}
                                                                                    }
                                                                                ],
                                                                                "characters": "Neon Wasteland",
                                                                                "fontSize": 18,
                                                                                "fontName": {
                                                                                    "family": "Inter",
                                                                                    "style": "Black"
                                                                                },
                                                                                "textAlignHorizontal": "LEFT",
                                                                                "textAlignVertical": "TOP",
                                                                                "lineHeight": {
                                                                                    "unit": "PIXELS",
                                                                                    "value": 27
                                                                                },
                                                                                "tailwind": {
                                                                                    "width": "w-148",
                                                                                    "height": "h-27",
                                                                                    "font-size": "text-18"
                                                                                }
                                                                            }
                                                                        ],
                                                                        "layoutMode": "NONE",
                                                                        "primaryAxisSizingMode": "AUTO",
                                                                        "counterAxisSizingMode": "FIXED",
                                                                        "primaryAxisAlignItems": "MIN",
                                                                        "counterAxisAlignItems": "MIN",
                                                                        "tailwind": {
                                                                            "width": "w-183",
                                                                            "height": "h-27",
                                                                            "margin-left": "ml-16",
                                                                            "flex": "flex",
                                                                            "justify-content": "justify-start",
                                                                            "align-items": "items-start"
                                                                        }
                                                                    },
                                                                    {
                                                                        "name": "p",
                                                                        "type": "FRAME",
                                                                        "x": 16,
                                                                        "y": 47,
                                                                        "width": 183.328125,
                                                                        "height": 40,
                                                                        "children": [
                                                                            {
                                                                                "name": "Post-punk revival with synth-driven soundscapes.",
                                                                                "type": "TEXT",
                                                                                "x": 0,
                                                                                "y": -0.5,
                                                                                "width": 179,
                                                                                "height": 60,
                                                                                "fills": [
                                                                                    {
                                                                                        "type": "SOLID",
                                                                                        "visible": true,
                                                                                        "opacity": 1,
                                                                                        "blendMode": "NORMAL",
                                                                                        "color": {
                                                                                            "r": 0.4156862795352936,
                                                                                            "g": 0.4470588266849518,
                                                                                            "b": 0.5098039507865906
                                                                                        },
                                                                                        "boundVariables": {}
                                                                                    }
                                                                                ],
                                                                                "characters": "Post-punk revival with synth-driven soundscapes.",
                                                                                "fontSize": 14,
                                                                                "fontName": {
                                                                                    "family": "Inter",
                                                                                    "style": "Regular"
                                                                                },
                                                                                "textAlignHorizontal": "LEFT",
                                                                                "textAlignVertical": "TOP",
                                                                                "lineHeight": {
                                                                                    "unit": "PIXELS",
                                                                                    "value": 20
                                                                                },
                                                                                "tailwind": {
                                                                                    "width": "w-179",
                                                                                    "height": "h-60",
                                                                                    "font-size": "text-14"
                                                                                }
                                                                            }
                                                                        ],
                                                                        "layoutMode": "NONE",
                                                                        "primaryAxisSizingMode": "AUTO",
                                                                        "counterAxisSizingMode": "FIXED",
                                                                        "primaryAxisAlignItems": "MIN",
                                                                        "counterAxisAlignItems": "MIN",
                                                                        "tailwind": {
                                                                            "width": "w-183",
                                                                            "height": "h-40",
                                                                            "margin-left": "ml-16",
                                                                            "flex": "flex",
                                                                            "justify-content": "justify-start",
                                                                            "align-items": "items-start"
                                                                        }
                                                                    }
                                                                ],
                                                                "layoutMode": "VERTICAL",
                                                                "primaryAxisSizingMode": "FIXED",
                                                                "counterAxisSizingMode": "FIXED",
                                                                "primaryAxisAlignItems": "MIN",
                                                                "counterAxisAlignItems": "MIN",
                                                                "paddingTop": 16,
                                                                "paddingRight": 16,
                                                                "paddingLeft": 16,
                                                                "itemSpacing": 4,
                                                                "tailwind": {
                                                                    "width": "w-215",
                                                                    "height": "h-103",
                                                                    "padding": "px-16 pt-16",
                                                                    "flex": "flex flex-col gap-4",
                                                                    "justify-content": "justify-start",
                                                                    "align-items": "items-start"
                                                                }
                                                            }
                                                        ],
                                                        "layoutMode": "VERTICAL",
                                                        "primaryAxisSizingMode": "FIXED",
                                                        "counterAxisSizingMode": "FIXED",
                                                        "primaryAxisAlignItems": "MIN",
                                                        "counterAxisAlignItems": "MIN",
                                                        "tailwind": {
                                                            "width": "w-215",
                                                            "height": "h-318",
                                                            "margin-left": "ml-479",
                                                            "flex": "flex flex-col",
                                                            "justify-content": "justify-start",
                                                            "align-items": "items-start"
                                                        }
                                                    }
                                                ],
                                                "layoutMode": "NONE",
                                                "primaryAxisSizingMode": "AUTO",
                                                "counterAxisSizingMode": "FIXED",
                                                "primaryAxisAlignItems": "MIN",
                                                "counterAxisAlignItems": "MIN",
                                                "tailwind": {
                                                    "width": "w-694",
                                                    "height": "h-318",
                                                    "flex": "flex",
                                                    "justify-content": "justify-start",
                                                    "align-items": "items-start"
                                                }
                                            }
                                        ],
                                        "layoutMode": "VERTICAL",
                                        "primaryAxisSizingMode": "FIXED",
                                        "counterAxisSizingMode": "FIXED",
                                        "primaryAxisAlignItems": "MIN",
                                        "counterAxisAlignItems": "MIN",
                                        "itemSpacing": 24,
                                        "tailwind": {
                                            "width": "w-694",
                                            "height": "h-378",
                                            "margin-left": "ml-32",
                                            "flex": "flex flex-col gap-24",
                                            "justify-content": "justify-start",
                                            "align-items": "items-start"
                                        }
                                    },
                                    {
                                        "name": "section",
                                        "type": "FRAME",
                                        "x": 32,
                                        "y": 1082.3359375,
                                        "width": 694,
                                        "height": 168,
                                        "fills": [
                                            {
                                                "type": "SOLID",
                                                "visible": true,
                                                "opacity": 1,
                                                "blendMode": "NORMAL",
                                                "color": {
                                                    "r": 0.0941176488995552,
                                                    "g": 0.0941176488995552,
                                                    "b": 0.10588235408067703
                                                },
                                                "boundVariables": {}
                                            }
                                        ],
                                        "strokes": [
                                            {
                                                "type": "SOLID",
                                                "opacity": 1,
                                                "blendMode": "NORMAL",
                                                "color": "#f6339a"
                                            }
                                        ],
                                        "cornerRadius": 14,
                                        "children": [
                                            {
                                                "name": "blockquote",
                                                "type": "FRAME",
                                                "x": 52,
                                                "y": 48,
                                                "width": 594,
                                                "height": 32,
                                                "children": [
                                                    {
                                                        "name": "\"Music is the weapon of the future\"",
                                                        "type": "TEXT",
                                                        "x": 0,
                                                        "y": 0,
                                                        "width": 594,
                                                        "height": 32,
                                                        "fills": [
                                                            {
                                                                "type": "SOLID",
                                                                "visible": true,
                                                                "opacity": 1,
                                                                "blendMode": "NORMAL",
                                                                "color": {
                                                                    "r": 1,
                                                                    "g": 1,
                                                                    "b": 1
                                                                },
                                                                "boundVariables": {}
                                                            }
                                                        ],
                                                        "characters": "\"Music is the weapon of the future\"",
                                                        "fontSize": 24,
                                                        "fontName": {
                                                            "family": "Inter",
                                                            "style": "Black"
                                                        },
                                                        "textAlignHorizontal": "LEFT",
                                                        "textAlignVertical": "TOP",
                                                        "lineHeight": {
                                                            "unit": "PIXELS",
                                                            "value": 32
                                                        },
                                                        "tailwind": {
                                                            "width": "w-594",
                                                            "height": "h-32",
                                                            "font-size": "text-24"
                                                        }
                                                    }
                                                ],
                                                "layoutMode": "HORIZONTAL",
                                                "primaryAxisSizingMode": "FIXED",
                                                "counterAxisSizingMode": "FIXED",
                                                "primaryAxisAlignItems": "MIN",
                                                "counterAxisAlignItems": "MIN",
                                                "tailwind": {
                                                    "width": "w-594",
                                                    "height": "h-32",
                                                    "margin-left": "ml-52",
                                                    "flex": "flex",
                                                    "justify-content": "justify-start",
                                                    "align-items": "items-start"
                                                }
                                            },
                                            {
                                                "name": "p",
                                                "type": "FRAME",
                                                "x": 52,
                                                "y": 96,
                                                "width": 594,
                                                "height": 24,
                                                "children": [
                                                    {
                                                        "name": "— Fela Kuti",
                                                        "type": "TEXT",
                                                        "x": 0,
                                                        "y": -2,
                                                        "width": 85,
                                                        "height": 24,
                                                        "fills": [
                                                            {
                                                                "type": "SOLID",
                                                                "visible": true,
                                                                "opacity": 1,
                                                                "blendMode": "NORMAL",
                                                                "color": {
                                                                    "r": 0.6000000238418579,
                                                                    "g": 0.6313725709915161,
                                                                    "b": 0.686274528503418
                                                                },
                                                                "boundVariables": {}
                                                            }
                                                        ],
                                                        "characters": "— Fela Kuti",
                                                        "fontSize": 16,
                                                        "fontName": {
                                                            "family": "Inter",
                                                            "style": "Regular"
                                                        },
                                                        "textAlignHorizontal": "LEFT",
                                                        "textAlignVertical": "TOP",
                                                        "lineHeight": {
                                                            "unit": "PIXELS",
                                                            "value": 24
                                                        },
                                                        "tailwind": {
                                                            "width": "w-85",
                                                            "height": "h-24",
                                                            "font-size": "text-16"
                                                        }
                                                    }
                                                ],
                                                "layoutMode": "NONE",
                                                "primaryAxisSizingMode": "AUTO",
                                                "counterAxisSizingMode": "FIXED",
                                                "primaryAxisAlignItems": "MIN",
                                                "counterAxisAlignItems": "MIN",
                                                "tailwind": {
                                                    "width": "w-594",
                                                    "height": "h-24",
                                                    "margin-left": "ml-52",
                                                    "flex": "flex",
                                                    "justify-content": "justify-start",
                                                    "align-items": "items-start"
                                                }
                                            }
                                        ],
                                        "layoutMode": "VERTICAL",
                                        "primaryAxisSizingMode": "FIXED",
                                        "counterAxisSizingMode": "FIXED",
                                        "primaryAxisAlignItems": "MIN",
                                        "counterAxisAlignItems": "MIN",
                                        "paddingTop": 48,
                                        "paddingRight": 48,
                                        "paddingLeft": 52,
                                        "itemSpacing": 16,
                                        "tailwind": {
                                            "width": "w-694",
                                            "height": "h-168",
                                            "margin-left": "ml-32",
                                            "padding": "pl-52 pr-48 pt-48",
                                            "flex": "flex flex-col gap-16",
                                            "justify-content": "justify-start",
                                            "align-items": "items-start"
                                        }
                                    }
                                ],
                                "layoutMode": "VERTICAL",
                                "primaryAxisSizingMode": "FIXED",
                                "counterAxisSizingMode": "FIXED",
                                "primaryAxisAlignItems": "MIN",
                                "counterAxisAlignItems": "MIN",
                                "paddingTop": 32,
                                "paddingRight": 32,
                                "paddingLeft": 32,
                                "itemSpacing": 48,
                                "tailwind": {
                                    "width": "w-758",
                                    "height": "h-1282",
                                    "padding": "px-32 pt-32",
                                    "flex": "flex flex-col gap-48",
                                    "justify-content": "justify-start",
                                    "align-items": "items-start"
                                }
                            }
                        ],
                        "layoutMode": "VERTICAL",
                        "primaryAxisSizingMode": "FIXED",
                        "counterAxisSizingMode": "FIXED",
                        "primaryAxisAlignItems": "MIN",
                        "counterAxisAlignItems": "MIN",
                        "paddingRight": 15,
                        "tailwind": {
                            "width": "w-773",
                            "height": "h-812",
                            "margin-left": "ml-256",
                            "padding": "pr-15",
                            "flex": "flex flex-col",
                            "justify-content": "justify-start",
                            "align-items": "items-start"
                        }
                    }
                ],
                "layoutMode": "HORIZONTAL",
                "primaryAxisSizingMode": "FIXED",
                "counterAxisSizingMode": "FIXED",
                "primaryAxisAlignItems": "MIN",
                "counterAxisAlignItems": "MIN",
                "tailwind": {
                    "width": "w-1029",
                    "height": "h-812",
                    "flex": "flex",
                    "justify-content": "justify-start",
                    "align-items": "items-start"
                }
            }
        ],
        "layoutMode": "VERTICAL",
        "primaryAxisSizingMode": "FIXED",
        "counterAxisSizingMode": "FIXED",
        "primaryAxisAlignItems": "MIN",
        "counterAxisAlignItems": "MIN",
        "tailwind": {
            "width": "w-1029",
            "height": "h-812",
            "flex": "flex flex-col",
            "justify-content": "justify-start",
            "align-items": "items-start"
        }
    }
]"##;

    pub fn document() -> Result<ImportedFigmaDocument, FigmaImportError> {
        import_figma_document(FIGMA_JSON)
    }

    pub fn runtime() -> Result<FigmaRuntime, FigmaRuntimeError> {
        FigmaRuntime::from_figma_json(FIGMA_JSON)
    }
}
