固定参数

```rust

pub init(cx: &mut App) {
    let command = Command::new("clipboard.open.view", "Clipboard");
    CommandRegister::register(command, |ctx: Context, cx: &mut App| {

        // 在现有窗口添加
        workspace::with_id_or_primary_workspace(ctx.workspace_id, cx, |workspace, cx| {
            let item = cx.new(|_| ClipboardItem::new());
            // 在主窗口打开
            workspace.open_item(item, cx);
            // 操作侧边栏
            workspace.active_pane().update(cx, |pane, cx| {
                // 激活条目
                pane.activate_item(item, cx);
                // 添加到侧边窗口
                pane.add_item(item, cx);
            })
        });

        // 使用单独的窗口
        workspace::open_workspace(open_options, cx, |workspace, cx| {
            let item = cx.new(|_| ClipboardItem::new());
            // 在主窗口打开
            workspace.open_item(item, cx);
        });
    }, cx);
}

```

动态参数

```rust

pub init(cx: &mut App) {
    let command = Command::new("clipboard.open.view", "Clipboard");
    CommandRegister::register(command, |ctx: Args, cx: &mut App| {}, cx);
}

```