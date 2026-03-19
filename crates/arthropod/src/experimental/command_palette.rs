#[cfg(feature = "nova")]
use flux_state::{Computed, ReadSignal, Runtime, Signal, WriteSignal};
#[cfg(feature = "nova")]
use std::sync::Arc;
#[cfg(feature = "nova")]
use widget_core::{Center, Column, Stack, Text, TextInput, Widget, WidgetContext, list_from};

/// A command that can be executed from the palette.
#[cfg(feature = "nova")]
#[derive(Clone)]
pub struct Command {
    pub id: String,
    pub label: String,
    /// Pre-calculated lowercased label for faster search filtering
    pub label_lowercase: String,
    pub shortcut: Option<String>,
    pub action: Arc<dyn Fn() + Send + Sync>,
}

#[cfg(feature = "nova")]
impl Command {
    pub fn new(
        id: impl Into<String>,
        label: impl Into<String>,
        action: impl Fn() + Send + Sync + 'static,
    ) -> Self {
        let label_str = label.into();
        Self {
            id: id.into(),
            label_lowercase: label_str.to_lowercase(),
            label: label_str,
            shortcut: None,
            action: Arc::new(action),
        }
    }

    pub fn shortcut(mut self, shortcut: impl Into<String>) -> Self {
        self.shortcut = Some(shortcut.into());
        self
    }
}

/// Registry for managing available commands.
#[cfg(feature = "nova")]
#[derive(Clone)]
pub struct CommandRegistry {
    #[allow(dead_code)]
    commands: Signal<Vec<Command>>,
    read: ReadSignal<Vec<Command>>,
    write: WriteSignal<Vec<Command>>,
}

#[cfg(feature = "nova")]
impl CommandRegistry {
    pub fn new(runtime: Arc<Runtime>) -> Self {
        let commands = Signal::new(runtime, Vec::new());
        let (read, write) = commands.clone().split();
        Self {
            commands,
            read,
            write,
        }
    }

    pub fn register(&self, command: Command) {
        self.write.update(|cmds| cmds.push(command));
    }

    pub fn unregister(&self, id: &str) {
        self.write.update(|cmds| {
            if let Some(pos) = cmds.iter().position(|c| c.id == id) {
                cmds.remove(pos);
            }
        });
    }

    pub fn read(&self) -> ReadSignal<Vec<Command>> {
        self.read.clone()
    }
}

/// The Command Palette widget itself.
#[cfg(feature = "nova")]
pub struct CommandPalette {
    registry: CommandRegistry,
    #[allow(dead_code)]
    visible: ReadSignal<bool>,
    close_action: Arc<dyn Fn() + Send + Sync>,
    runtime: Arc<Runtime>,
}

#[cfg(feature = "nova")]
impl CommandPalette {
    pub fn new(
        registry: CommandRegistry,
        visible: ReadSignal<bool>,
        close_action: impl Fn() + Send + Sync + 'static,
        runtime: Arc<Runtime>,
    ) -> Self {
        Self {
            registry,
            visible,
            close_action: Arc::new(close_action),
            runtime,
        }
    }
}

#[cfg(feature = "nova")]
impl Widget for CommandPalette {
    fn build(&self, ctx: &mut WidgetContext) -> render_engine::NodeId {
        let query = Signal::new(self.runtime.clone(), String::new());
        let (query_read, _) = query.clone().split();

        // Filtered commands
        let registry_read = self.registry.read();
        let query_read_clone = query_read.clone();

        let filtered_commands = Computed::new(self.runtime.clone(), move || {
            let q = query_read_clone.get().to_lowercase();
            let cmds = registry_read.get();
            if q.is_empty() {
                cmds
            } else {
                cmds.into_iter()
                    .filter(|c| c.label_lowercase.contains(&q))
                    .collect()
            }
        });

        // The Palette UI
        let close = self.close_action.clone();

        let palette_content = Column::new((
            // Search Bar
            TextInput::new(query)
                .placeholder("Type a command...")
                .padding(12.0)
                .width(400.0),
            // List of commands
            {
                let mut slots = Vec::new();
                for i in 0..5 {
                    let fc = filtered_commands.clone();
                    let close_cmd = close.clone();

                    // Computed label for slot i
                    let label_sig = Computed::new(self.runtime.clone(), move || {
                        let cmds = fc.get();
                        if i < cmds.len() {
                            cmds[i].label.clone()
                        } else {
                            String::new()
                        }
                    });

                    let fc_action = filtered_commands.clone();
                    let close_action = close_cmd.clone();

                    struct ReactiveButton {
                        label: Computed<String>,
                        action: Arc<dyn Fn() + Send + Sync>,
                    }

                    impl Widget for ReactiveButton {
                        fn build(&self, ctx: &mut WidgetContext) -> render_engine::NodeId {
                            let node = ctx.create_node(
                                ctx.root(),
                                render_engine::NodeContent::Styled {
                                    style: Box::new(
                                        render_engine::VisualStyle::new()
                                            .solid_fill(
                                                render_engine::Color::rgba(0.2, 0.2, 0.2, 1.0)
                                                    .as_vec4(),
                                            )
                                            .corner_radius(4.0),
                                    ),
                                },
                            );

                            let txt = Text::computed(self.label.clone())
                                .color(render_engine::Color::WHITE);
                            let txt_id = txt.build(ctx);
                            ctx.reparent_node(txt_id, ctx.root(), node);

                            ctx.set_layout_style(
                                node,
                                layout_engine::FlexStyle {
                                    padding_left: 12.0,
                                    padding_right: 12.0,
                                    padding_top: 8.0,
                                    padding_bottom: 8.0,
                                    ..Default::default()
                                },
                            );

                            ctx.add_clickable(node, self.action.clone());
                            ctx.add_hover_state(node);

                            node
                        }
                    }

                    let btn = ReactiveButton {
                        label: label_sig,
                        action: Arc::new(move || {
                            let cmds = fc_action.get();
                            if i < cmds.len() {
                                (cmds[i].action)();
                                close_action();
                            }
                        }),
                    };

                    slots.push(btn);
                }

                list_from(slots)
            },
        ))
        .gap(8.0)
        .padding(16.0);

        // Center the palette on screen
        let centered = Center::new(palette_content);

        // Add a backdrop
        Stack::new((Backdrop, centered)).build(ctx)
    }
}

// Helper widget for backdrop
#[cfg(feature = "nova")]
struct Backdrop;

#[cfg(feature = "nova")]
impl Widget for Backdrop {
    fn build(&self, ctx: &mut WidgetContext) -> render_engine::NodeId {
        let node = ctx.create_node(
            ctx.root(),
            render_engine::NodeContent::Styled {
                style: Box::new(
                    render_engine::VisualStyle::new()
                        .solid_fill(render_engine::Color::rgba(0.0, 0.0, 0.0, 0.5).as_vec4()),
                ),
            },
        );

        ctx.set_layout_style(
            node,
            layout_engine::FlexStyle {
                flex_grow: 1.0,
                width: Some(10000.0), // Hack to ensure full coverage if flex fails
                height: Some(10000.0),
                ..Default::default()
            },
        );

        node
    }
}

/// A convenience wrapper that places the palette over your app.
#[cfg(feature = "nova")]
pub struct CommandPaletteOverlay<W: Widget> {
    content: W,
    palette: CommandPalette,
}

#[cfg(feature = "nova")]
impl<W: Widget> CommandPaletteOverlay<W> {
    pub fn new(content: W, palette: CommandPalette) -> Self {
        Self { content, palette }
    }
}

#[cfg(feature = "nova")]
impl<W: Widget> Widget for CommandPaletteOverlay<W> {
    fn build(&self, ctx: &mut WidgetContext) -> render_engine::NodeId {
        let root = ctx.create_node(ctx.root(), render_engine::NodeContent::Empty);

        // Build content (bottom layer)
        let content_id = self.content.build(ctx);
        ctx.reparent_to(content_id, root);

        // Build palette (top layer)
        let palette_id = self.palette.build(ctx);
        ctx.reparent_to(palette_id, root);

        // Stack layout
        ctx.set_layout_style(
            root,
            layout_engine::FlexStyle {
                flex_grow: 1.0,
                ..Default::default()
            },
        );

        for id in [content_id, palette_id] {
            ctx.set_layout_style(
                id,
                layout_engine::FlexStyle {
                    flex_grow: 1.0,
                    width: Some(0.0),
                    height: Some(0.0),
                    ..Default::default()
                },
            );
        }

        root
    }
}

#[cfg(test)]
#[cfg(feature = "nova")]
mod tests {
    use super::*;
    use flux_state::Runtime;

    #[test]
    fn test_registry() {
        let rt = Runtime::new();
        let registry = CommandRegistry::new(rt.clone());

        let cmd = Command::new("test", "Test Command", || {});
        registry.register(cmd);

        let cmds = registry.read().get();
        assert_eq!(cmds.len(), 1);
        assert_eq!(cmds[0].id, "test");

        registry.unregister("test");
        let cmds = registry.read().get();
        assert_eq!(cmds.len(), 0);
    }

    #[test]
    fn test_search_filtering() {
        let rt = Runtime::new();
        let registry = CommandRegistry::new(rt.clone());

        let cmd1 = Command::new("test1", "Apple", || {});
        let cmd2 = Command::new("test2", "Banana", || {});

        registry.register(cmd1);
        registry.register(cmd2);

        let query = Signal::new(rt.clone(), "App".to_string());
        let (query_read, _) = query.split();
        let registry_read = registry.read();

        let filtered = Computed::new(rt.clone(), move || {
            let q = query_read.get().to_lowercase();
            let cmds = registry_read.get();
            cmds.into_iter()
                .filter(|c| c.label_lowercase.contains(&q))
                .collect::<Vec<_>>()
        });

        let results = filtered.get();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].label, "Apple");
    }
}
