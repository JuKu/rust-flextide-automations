# Markdown WYSIWYG Editor Research

## Overview

Research into open-source Next.js/React Markdown WYSIWYG editors with real-time collaboration capabilities, custom block support, and comprehensive toolbars. Focus on solutions that are fully open-source and free to use.

## Requirements

### Core Requirements
- Open-source (free, no paid tiers)
- Next.js/React compatible
- Markdown WYSIWYG editing
- Real-time collaboration
- Custom Markdown blocks/code support
- Toolbar with formatting buttons (bold, italic, headlines, bullet lists, code blocks)

### Nice-to-Have Features
- Slash commands (Notion-style)
- Bubble menu (popup when selecting text)
- Floating menu (similar to slash commands)
- Confluence/Notion-inspired UX

## Primary Candidates

### 1. Tiptap (Recommended)

**Overview:**
Tiptap is a headless, framework-agnostic WYSIWYG editor built on ProseMirror. It's fully open-source (MIT license) and offers extensive customization capabilities.

**Key Features:**
- ✅ Fully open-source (MIT license)
- ✅ React/Next.js integration via `@tiptap/react`
- ✅ Markdown support via `@tiptap/extension-markdown`
- ✅ Real-time collaboration via Yjs integration
- ✅ Highly extensible architecture for custom blocks
- ✅ Headless design (full UI control)
- ✅ Slash commands support (via extensions)
- ✅ Bubble menu support (via `@tiptap/extension-bubble-menu`)
- ✅ Floating menu support (via `@tiptap/extension-floating-menu`)

**Installation:**
```bash
pnpm add @tiptap/react @tiptap/pm @tiptap/starter-kit @tiptap/extension-markdown
```

**Real-Time Collaboration Setup:**
```bash
pnpm add yjs y-websocket @tiptap/extension-collaboration @tiptap/extension-collaboration-cursor
```

**Backend Options:**

**Node.js/TypeScript Options:**
- **Hocuspocus**: Open-source WebSocket backend for Yjs, can be self-hosted
  ```bash
  pnpm add @hocuspocus/server @hocuspocus/provider
  ```
- **y-websocket**: Simple WebSocket provider (requires custom server implementation)

**Rust Backend Options (Recommended for Rust-only backends):**
- **yrs-warp**: WebSocket server implementation for Yrs (Rust port of Yjs) using Warp framework
  - GitHub: https://github.com/y-crdt/yrs-warp
  - Fully Rust-based, no Node.js required
  - Compatible with Yjs clients (Tiptap)
  
- **Y-CRDT (yrs) + Custom WebSocket**: Build custom WebSocket server using `yrs` crate with `tokio-tungstenite` or `axum`
  - GitHub: https://github.com/y-crdt/y-crdt
  - Full control over implementation
  - Requires implementing Yjs WebSocket protocol

- **Y-Sweet**: Open-source suite built on Yrs, can be deployed as WebAssembly on Cloudflare
  - Built on Yrs (Rust)
  - Can be integrated into Rust backends
  - Supports edge deployment

**Basic Setup Example (Node.js Backend with Hocuspocus):**
```typescript
import { useEditor, EditorContent } from '@tiptap/react';
import StarterKit from '@tiptap/starter-kit';
import Markdown from '@tiptap/extension-markdown';
import Collaboration from '@tiptap/extension-collaboration';
import CollaborationCursor from '@tiptap/extension-collaboration-cursor';
import * as Y from 'yjs';
import { HocuspocusProvider } from '@hocuspocus/provider';

const ydoc = new Y.Doc();
const provider = new HocuspocusProvider({
  url: 'ws://localhost:1234',
  name: 'document-id',
  document: ydoc,
});

const editor = useEditor({
  extensions: [
    StarterKit.configure({
      history: false, // Disable default history for collaboration
    }),
    Markdown,
    Collaboration.configure({
      document: ydoc,
    }),
    CollaborationCursor.configure({
      provider,
    }),
  ],
});

return <EditorContent editor={editor} />;
```

**Basic Setup Example (Rust Backend with yrs-warp or custom):**
```typescript
import { useEditor, EditorContent } from '@tiptap/react';
import StarterKit from '@tiptap/starter-kit';
import Markdown from '@tiptap/extension-markdown';
import Collaboration from '@tiptap/extension-collaboration';
import CollaborationCursor from '@tiptap/extension-collaboration-cursor';
import * as Y from 'yjs';
import { WebsocketProvider } from 'y-websocket';

const ydoc = new Y.Doc();
// Connect to your Rust WebSocket server
const provider = new WebsocketProvider(
  'ws://localhost:3030/collaboration', // Your Rust server URL
  'document-id',
  ydoc
);

const editor = useEditor({
  extensions: [
    StarterKit.configure({
      history: false, // Disable default history for collaboration
    }),
    Markdown,
    Collaboration.configure({
      document: ydoc,
    }),
    CollaborationCursor.configure({
      provider,
    }),
  ],
});

return <EditorContent editor={editor} />;
```

**Toolbar Implementation:**
Tiptap is headless, so you build your own toolbar using editor commands:
```typescript
<button onClick={() => editor.chain().focus().toggleBold().run()}>
  Bold
</button>
<button onClick={() => editor.chain().focus().toggleHeading({ level: 1 }).run()}>
  H1
</button>
```

**Custom Blocks:**
Create custom extensions by extending Tiptap's Node or Mark classes:
```typescript
import { Node } from '@tiptap/core';

export const CustomBlock = Node.create({
  name: 'customBlock',
  // Define your custom block behavior
});
```

**Slash Commands:**
Implement via custom extension or use community extensions. Requires listening to text input and showing a menu.

**Bubble Menu:**
```bash
pnpm add @tiptap/extension-bubble-menu
```

**Floating Menu:**
```bash
pnpm add @tiptap/extension-floating-menu
```

**Pros:**
- Fully open-source and free
- Excellent documentation
- Active community
- Highly extensible
- Real-time collaboration via Yjs (proven technology)
- Self-hostable collaboration backend (Hocuspocus)
- Built on ProseMirror (mature, battle-tested)

**Cons:**
- Headless design requires more UI development
- Steeper learning curve
- Requires backend setup for collaboration (Hocuspocus or custom)

**License:** MIT (fully open-source)

**Documentation:** https://tiptap.dev

---

### 2. BlockNote

**Overview:**
BlockNote is an open-source, block-based rich text editor designed for React. It provides a Notion-style experience out of the box.

**Key Features:**
- ✅ Open-source (MIT license)
- ✅ React integration
- ✅ Block-based architecture (similar to Notion)
- ✅ Built-in UI components (toolbar, menus)
- ✅ Real-time collaboration support
- ✅ Custom blocks support
- ✅ Slash commands built-in
- ✅ Markdown import/export

**Installation:**
```bash
pnpm add @blocknote/core @blocknote/react
```

**Real-Time Collaboration:**
Uses Yjs under the hood, similar to Tiptap.

**Pros:**
- Notion-like UX out of the box
- Built-in UI components (less custom development)
- Slash commands included
- Block-based architecture (good for custom blocks)
- Active development

**Cons:**
- Less flexible than Tiptap (more opinionated)
- Smaller community than Tiptap
- May require more customization for specific needs
- Less documentation/examples available

**License:** MIT (fully open-source)

**Documentation:** https://www.blocknotejs.org

---

### 3. MDXEditor

**Overview:**
MDXEditor is an open-source React component built on Lexical (Facebook's editor framework) that provides WYSIWYG Markdown editing.

**Key Features:**
- ✅ Open-source
- ✅ React integration
- ✅ Built on Lexical (Facebook's editor framework)
- ✅ Markdown-focused
- ✅ Customizable toolbar
- ✅ Plugin architecture
- ✅ Code blocks with syntax highlighting

**Installation:**
```bash
pnpm add @mdxeditor/editor
```

**Real-Time Collaboration:**
- ❌ No native real-time collaboration support
- Would require custom implementation with Yjs or similar

**Pros:**
- Built on Lexical (modern, performant)
- Markdown-first approach
- Good plugin system
- Syntax highlighting for code blocks

**Cons:**
- No built-in real-time collaboration
- Less mature than Tiptap
- Smaller ecosystem

**License:** Open-source (check specific license)

**Documentation:** https://mdxeditor.dev

---

### 4. React MDEditor (@uiw/react-md-editor)

**Overview:**
Simple Markdown editor with preview, implemented with React.js and TypeScript.

**Key Features:**
- ✅ Open-source
- ✅ React/TypeScript
- ✅ Next.js compatible (with dynamic import)
- ✅ Toolbar with formatting options
- ✅ Preview mode

**Installation:**
```bash
pnpm add @uiw/react-md-editor
```

**Real-Time Collaboration:**
- ❌ No native support
- Would require significant custom development

**Pros:**
- Simple and lightweight
- Easy to integrate
- Good for basic Markdown editing

**Cons:**
- No real-time collaboration
- Limited customization
- Less feature-rich than alternatives
- Not as modern/extensible

**License:** MIT

**Documentation:** https://github.com/uiwjs/react-md-editor

---

## Comparison Matrix

| Feature | Tiptap | BlockNote | MDXEditor | React MDEditor |
|---------|--------|-----------|-----------|----------------|
| Open Source | ✅ MIT | ✅ MIT | ✅ | ✅ MIT |
| Real-Time Collaboration | ✅ (Yjs) | ✅ (Yjs) | ❌ | ❌ |
| Custom Blocks | ✅ | ✅ | ⚠️ (via plugins) | ❌ |
| Toolbar | ⚠️ (custom) | ✅ (built-in) | ✅ | ✅ |
| Slash Commands | ⚠️ (extension) | ✅ (built-in) | ❌ | ❌ |
| Bubble Menu | ✅ | ✅ | ❌ | ❌ |
| Markdown Support | ✅ | ✅ | ✅ | ✅ |
| Next.js Compatible | ✅ | ✅ | ✅ | ✅ (with dynamic) |
| Learning Curve | Medium-High | Low-Medium | Medium | Low |
| Community Size | Large | Medium | Small | Medium |
| Documentation | Excellent | Good | Good | Basic |

## Recommendation

### Primary Recommendation: **Tiptap**

**Rationale:**
1. **Fully Open-Source:** MIT license, no paid tiers required
2. **Real-Time Collaboration:** Proven Yjs integration with self-hostable backend (Hocuspocus)
3. **Extensibility:** Best-in-class for custom blocks and extensions
4. **Mature Ecosystem:** Large community, excellent documentation, battle-tested
5. **Flexibility:** Headless design allows complete UI control
6. **All Required Features:** Supports all core requirements and nice-to-haves

**Trade-offs:**
- Requires more initial setup (toolbar, menus)
- Steeper learning curve
- Need to set up collaboration backend (Hocuspocus)

### Alternative: **BlockNote**

**When to Consider:**
- Want Notion-like UX out of the box
- Prefer built-in UI components
- Need faster initial development
- Less customization needed

**Trade-offs:**
- Less flexible than Tiptap
- Smaller community/ecosystem
- May hit limitations for advanced use cases

## Implementation Notes

### Tiptap Setup Checklist

1. **Core Packages:**
   ```bash
   pnpm add @tiptap/react @tiptap/pm @tiptap/starter-kit @tiptap/extension-markdown
   ```

2. **Collaboration Packages:**
   ```bash
   pnpm add yjs @tiptap/extension-collaboration @tiptap/extension-collaboration-cursor
   ```

3. **Backend Options:**
   - **Node.js (Hocuspocus - Self-Hosted):**
     ```bash
     pnpm add @hocuspocus/server @hocuspocus/provider
     ```
   - **Rust (yrs-warp or custom):** See Rust Backend Setup section below

4. **UI Extensions (Nice-to-Have):**
   ```bash
   pnpm add @tiptap/extension-bubble-menu @tiptap/extension-floating-menu
   ```

5. **Additional Useful Extensions:**
   ```bash
   pnpm add @tiptap/extension-placeholder @tiptap/extension-focus
   ```

### Hocuspocus Backend Setup

Hocuspocus is the recommended self-hosted WebSocket backend for Tiptap collaboration:

```typescript
// server.ts
import { Server } from '@hocuspocus/server';

const server = new Server({
  port: 1234,
  // Add authentication, persistence, etc.
});

server.listen();
```

**Features:**
- Open-source
- Self-hostable
- Supports authentication
- Supports persistence
- Production-ready

### Rust Backend Setup (Alternative to Hocuspocus)

For Rust-only backends, you have several options:

#### Option 1: yrs-warp (Recommended)

**yrs-warp** is a WebSocket server implementation for Yrs (Rust port of Yjs) using the Warp framework.

**Installation:**
```toml
# Cargo.toml
[dependencies]
yrs = "0.19"
yrs-warp = "0.1"
warp = "0.3"
tokio = { version = "1", features = ["full"] }
```

**Basic Setup:**
```rust
use yrs_warp::YrsWarpServer;
use warp::Filter;

#[tokio::main]
async fn main() {
    let server = YrsWarpServer::new();
    
    let routes = warp::path("collaboration")
        .and(warp::ws())
        .map(|ws: warp::ws::Ws| {
            // Handle WebSocket connections
            ws.on_upgrade(|websocket| {
                server.handle_connection(websocket)
            })
        });
    
    warp::serve(routes)
        .run(([127, 0, 0, 1], 3030))
        .await;
}
```

**Features:**
- Fully Rust-based
- Compatible with Yjs clients (Tiptap works seamlessly)
- Uses Warp (lightweight, fast)
- Open-source

**GitHub:** https://github.com/y-crdt/yrs-warp

#### Option 2: Custom Implementation with yrs + tokio-tungstenite

Build a custom WebSocket server using `yrs` and `tokio-tungstenite`:

**Installation:**
```toml
[dependencies]
yrs = "0.19"
tokio = { version = "1", features = ["full"] }
tokio-tungstenite = "0.21"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

**Implementation Overview:**
1. Create WebSocket server with `tokio-tungstenite`
2. Use `yrs` to manage Yjs documents
3. Implement Yjs WebSocket protocol:
   - Handle sync messages (document updates)
   - Handle awareness messages (user presence, cursors)
   - Broadcast updates to all connected clients
4. Optionally persist documents to database

**Key Components:**
- Document state management with `yrs::Doc`
- WebSocket message handling (sync protocol)
- Awareness protocol for user presence
- Client connection management

**Resources:**
- Yjs Protocol: https://github.com/yjs/y-protocols
- yrs Documentation: https://docs.rs/yrs/

#### Option 3: Y-Sweet

**Y-Sweet** is an open-source suite built on Yrs that can be deployed as WebAssembly on Cloudflare or integrated into Rust backends.

**Features:**
- Built on Yrs (Rust)
- Can be integrated into existing Rust applications
- Supports edge deployment (Cloudflare Workers)
- Open-source

**GitHub:** https://github.com/y-sweet/y-sweet

#### Frontend Integration with Rust Backend

The frontend setup remains the same - Tiptap with Yjs works with any Yjs-compatible WebSocket server:

```typescript
import { WebsocketProvider } from 'y-websocket';
import * as Y from 'yjs';

const ydoc = new Y.Doc();
const provider = new WebsocketProvider(
  'ws://localhost:3030/collaboration', // Your Rust WebSocket server
  'document-id',
  ydoc
);

// Use with Tiptap as before
const editor = useEditor({
  extensions: [
    StarterKit,
    Collaboration.configure({ document: ydoc }),
    CollaborationCursor.configure({ provider }),
  ],
});
```

**Important Notes:**
- Yjs WebSocket protocol is language-agnostic
- Any backend implementing the Yjs protocol will work with Tiptap
- `yrs` (Rust) and `yjs` (JavaScript) are fully compatible
- You need to implement:
  - Document sync protocol
  - Awareness protocol (for cursors/presence)
  - Connection management

### Custom Blocks Implementation

Tiptap allows creating custom nodes/marks:

```typescript
import { Node } from '@tiptap/core';

export const CustomMarkdownBlock = Node.create({
  name: 'customMarkdownBlock',
  group: 'block',
  parseHTML() {
    return [{ tag: 'div[data-type="custom-block"]' }];
  },
  renderHTML({ HTMLAttributes }) {
    return ['div', { 'data-type': 'custom-block', ...HTMLAttributes }, 0];
  },
  addMarkdownRules() {
    return [
      {
        find: /^:::custom\s+(.+)$/,
        type: this.type,
        getAttributes: (match) => ({
          content: match[1],
        }),
      },
    ];
  },
});
```

### Slash Commands Implementation

Slash commands can be implemented by:
1. Listening to text input (e.g., "/")
2. Showing a menu with available commands
3. Inserting the selected block/formatting

Example approach:
- Use Tiptap's `onUpdate` or `onTransaction` hooks
- Detect "/" at start of line
- Show floating menu with options
- Replace "/" and selected option with actual content

### Next.js Integration

Tiptap works with Next.js, but may require dynamic imports for SSR:

```typescript
import dynamic from 'next/dynamic';

const Editor = dynamic(() => import('./Editor'), { ssr: false });
```

Or configure Tiptap to work with SSR by ensuring proper hydration.

## Security Considerations

1. **XSS Protection:** Tiptap/ProseMirror sanitizes HTML by default
2. **Collaboration:** Ensure proper authentication on Hocuspocus server
3. **Custom Blocks:** Validate and sanitize custom block content
4. **Markdown Parsing:** Use trusted Markdown parsers

## Performance Considerations

1. **Large Documents:** Tiptap/ProseMirror handles large documents well
2. **Collaboration:** Yjs is efficient for real-time sync
3. **Bundle Size:** Tiptap is modular (tree-shakeable)
4. **Rendering:** ProseMirror uses virtual DOM-like updates

## Resources

### Tiptap
- **Documentation:** https://tiptap.dev
- **GitHub:** https://github.com/ueberdosis/tiptap
- **Examples:** https://tiptap.dev/examples
- **Collaboration Guide:** https://tiptap.dev/docs/collaboration/getting-started/install
- **Hocuspocus:** https://github.com/ueberdosis/hocuspocus

### BlockNote
- **Documentation:** https://www.blocknotejs.org
- **GitHub:** https://github.com/TypeCellOS/BlockNote

### MDXEditor
- **Documentation:** https://mdxeditor.dev
- **GitHub:** https://github.com/mdx-editor/mdx-editor

### Yjs
- **Documentation:** https://docs.yjs.dev
- **GitHub:** https://github.com/yjs/yjs

### Rust Backend (Yrs)
- **yrs (Y-CRDT):** https://github.com/y-crdt/y-crdt
- **yrs-warp:** https://github.com/y-crdt/yrs-warp
- **yrs Documentation:** https://docs.rs/yrs/
- **Y-Sweet:** https://github.com/y-sweet/y-sweet
- **Yjs Protocol Spec:** https://github.com/yjs/y-protocols

## Conclusion

**Tiptap** is the recommended solution for this project due to:
- Complete open-source solution (no paid requirements)
- Full feature set (collaboration, custom blocks, extensibility)
- Mature ecosystem and documentation
- Self-hostable collaboration backend (Node.js or Rust)
- All required and nice-to-have features supported
- **Rust backend compatible** via yrs/yrs-warp (no Node.js required)

**For Rust-only backends:**
- Use **yrs-warp** for a ready-made WebSocket server solution
- Or implement custom WebSocket server with `yrs` + `tokio-tungstenite`/`axum`
- Yjs protocol is language-agnostic - any backend implementing it works with Tiptap
- `yrs` (Rust) and `yjs` (JavaScript) are fully compatible

The initial setup effort is justified by the flexibility and capabilities it provides. For a Confluence/Notion-like experience, Tiptap can be configured with custom UI components, slash commands, and floating menus to match the desired UX. The Rust backend integration via yrs ensures you can maintain a pure Rust stack without requiring Node.js servers.

