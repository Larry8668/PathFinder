# Screen Sharing Implementation Plan

## Overview
Build a WebRTC-based screen sharing system where:
- Tauri app hosts a local signaling server (WebSocket)
- React frontend connects to the server
- Code-based access: sharer generates a code, viewers enter code to connect
- One-to-many: one sharer can share to multiple viewers

## Architecture

```
┌─────────────────────────────────────────────┐
│         Tauri App (Rust Backend)            │
│  ┌───────────────────────────────────────┐  │
│  │  WebSocket Signaling Server           │  │
│  │  - Session management (code → session)│  │
│  │  - WebRTC signaling (offer/answer)    │  │
│  │  - ICE candidate relay                │  │
│  └───────────────────────────────────────┘  │
└─────────────────────────────────────────────┘
         ↑ WebSocket                ↑ WebSocket
         │                           │
┌────────┴─────────┐      ┌─────────┴──────────┐
│  React Frontend  │      │  React Frontend    │
│  (Sharer Tab)    │      │  (Viewer Tab)      │
│  - Screen capture│      │  - Display stream  │
│  - WebRTC offer  │      │  - WebRTC answer   │
└──────────────────┘      └────────────────────┘
         │                           │
         └─────────── WebRTC ────────┘
            (Peer-to-Peer Connection)
```

## Implementation Goals

### Goal 1: WebSocket Signaling Server
**What:** Set up a WebSocket server in Rust that handles connections and manages sessions.

**Deliverables:**
- WebSocket server on `localhost:8765`
- Session management with code generation
- Connection handling for sharer and viewer roles

**Test:** 
- Start server, verify it accepts WebSocket connections
- Generate a session code via command
- Connect two WebSocket clients with the same code

**Success Criteria:**
- Server starts without errors
- Can generate unique 6-character codes
- WebSocket connections are accepted and stored by session code

---

### Goal 2: Basic React UI for Screen Share Management
**What:** Add a "Screen Share" option in HomeOptions that opens a React page for managing screen sharing.

**Deliverables:**
- New option in `HomeOptions.jsx`: "Screen Share"
- New React component: `ScreenSharePage.jsx`
- UI for:
  - Starting/stopping a screen share session
  - Displaying the share code
  - Copying the share code

**Test:**
- Click "Screen Share" option from home
- See the Screen Share management page
- Click "Start Sharing" generates a code

**Success Criteria:**
- Navigation works
- UI displays correctly
- Code generation works (via Tauri command)

---

### Goal 3: WebSocket Connection in React
**What:** Connect React frontend to WebSocket server and handle basic message exchange.

**Deliverables:**
- WebSocket client connection in React
- Message sending/receiving
- Connection state management

**Test:**
- Open Screen Share page, verify WebSocket connects
- Send test message from React, verify server receives it
- Send test message from server, verify React receives it

**Success Criteria:**
- WebSocket connection established
- Messages can be sent/received in both directions
- Connection errors are handled gracefully

---

### Goal 4: Screen Capture and WebRTC Offer
**What:** Capture screen using `getDisplayMedia` and create WebRTC offer.

**Deliverables:**
- Screen capture using `getDisplayMedia` API
- Create `RTCPeerConnection` 
- Generate WebRTC offer with screen stream
- Send offer via WebSocket

**Test:**
- Click "Start Sharing", browser prompts for screen selection
- Select screen, verify local preview (optional)
- Verify WebRTC offer is created and sent via WebSocket

**Success Criteria:**
- Screen capture works (browser prompts)
- WebRTC offer is generated
- Offer is sent to server via WebSocket
- No errors in console

---

### Goal 5: Viewer Connection and WebRTC Answer
**What:** Viewer connects via code, receives offer, creates answer, and establishes connection.

**Deliverables:**
- Viewer page/UI for entering code
- Viewer WebSocket connection with code
- Receive offer from server
- Create WebRTC answer
- Send answer via WebSocket
- Exchange ICE candidates

**Test:**
- Sharer starts sharing (Goal 4)
- Viewer enters code in viewer UI
- Viewer receives offer and creates answer
- Answer is sent back to sharer
- Both sides exchange ICE candidates

**Success Criteria:**
- Viewer can connect with code
- Offer/answer exchange works
- ICE candidates are exchanged
- WebRTC connection state progresses to "connected"

---

### Goal 6: Display Shared Screen
**What:** Display the received screen stream in viewer's video element.

**Deliverables:**
- Handle `ontrack` event
- Display stream in `<video>` element
- Handle video playback (autoplay restrictions)

**Test:**
- Complete Goals 4 and 5
- Verify viewer sees the shared screen
- Verify screen updates in real-time

**Success Criteria:**
- Viewer sees the shared screen
- Video plays automatically or with user interaction
- Stream quality is acceptable
- Connection is stable

---

## Technical Decisions

### WebSocket Library
- **Rust:** Use `tokio-tungstenite` with `axum` for WebSocket support
- **React:** Use native `WebSocket` API (or `react-use-websocket` if needed)

### WebRTC Configuration
- **For localhost:** Use minimal ICE servers (empty or localhost STUN)
- **Code generation:** 6-character alphanumeric (uppercase)
- **Session management:** Store in `Arc<Mutex<HashMap<Code, Session>>>`

### Message Protocol
All WebSocket messages are JSON:
```json
{
  "type": "offer" | "answer" | "ice-candidate" | "join" | "error",
  "code": "ABC123",
  "data": { ... }  // SDP or ICE candidate
}
```

---

## File Structure

```
src-tauri/src/lib.rs
  - WebSocket handler
  - Session management
  - Tauri commands

src/components/ScreenSharePage.jsx
  - Sharer UI
  - WebSocket client
  - WebRTC logic

src/components/ViewerPage.jsx (optional, or part of ScreenSharePage)
  - Viewer UI
  - WebSocket client
  - WebRTC logic

src/components/HomeOptions.jsx
  - Add "Screen Share" option
```

---

## Step-by-Step Implementation Order

1. **Add dependencies** to `Cargo.toml` (WebSocket support)
2. **Goal 1:** Build WebSocket signaling server
3. **Goal 2:** Add React UI skeleton
4. **Goal 3:** Connect React to WebSocket
5. **Goal 4:** Implement screen capture and offer
6. **Goal 5:** Implement viewer connection and answer
7. **Goal 6:** Display stream and polish

---

## Questions to Decide

1. **Single page or two pages?**
   - Option A: One page with tabs/mode switcher (Sharer/Viewer)
   - Option B: Separate pages/URLs
   - **Recommendation:** Option A - simpler navigation

2. **Where to capture screen?**
   - Option A: In Tauri WebView (has limitations)
   - Option B: Open system browser for screen capture
   - **Recommendation:** Option B - more reliable

3. **Multiple viewers?**
   - Start with one viewer (simpler)
   - Add multiple viewers later if needed

---

## Next Steps

Once you approve this plan, I'll start with:
1. Adding dependencies
2. Implementing Goal 1 (WebSocket server)
3. Testing Goal 1 before moving forward

Does this plan look good to you? Any changes or questions?
