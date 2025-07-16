# Feynman Tutor

An AI-powered conversation system that helps users identify gaps in their understanding by having them teach concepts to an AI tutor named Feynman.

## Overview

The Feynman Tutor follows this conversation flow:
1. **Greeting**: AI asks what topic the user wants to teach
2. **Topic Acknowledgment**: AI confirms readiness to learn about the topic
3. **Teaching Phase**: User explains the concept while AI listens without interruption
4. **Analysis Phase**: AI analyzes the explanation for gaps in understanding
5. **Questioning Phase**: AI asks targeted questions to probe identified gaps
6. **Completion**: AI congratulates the user when all questions are answered well

## Architecture

- **Backend**: Rust + Axum + Tokio server with WebSocket support
- **Frontend**: React + TypeScript + Vite with Web Audio API
- **AI Integration**: OpenAI Realtime API for voice conversation

## Setup

### Prerequisites
- Rust (latest stable)
- Node.js 18+
- OpenAI API key (for production use)

### Development Setup

1. **Clone and setup backend:**
   ```bash
   cd backend
   cargo build
   ```

2. **Setup frontend:**
   ```bash
   cd front
   npm install
   npm run build
   ```

3. **Environment Variables:**
   Create a `.env` file in the backend directory:
   ```bash
   OPENAI_API_KEY=sk-your-openai-api-key-here
   ```

### Running the Application

#### Test Mode (No OpenAI API Key Required)
```bash
# Terminal 1: Start backend in test mode
cd backend
TEST_MODE=true cargo run

# Terminal 2: Start frontend dev server
cd front
npm run dev
```

#### Production Mode (OpenAI API Key Required)
```bash
# Terminal 1: Start backend with OpenAI API
cd backend
cargo run

# Terminal 2: Start frontend dev server  
cd front
npm run dev
```

Then open http://localhost:5173 in your browser.

## Usage

1. Click "Start Teaching" to begin
2. Grant microphone permissions when prompted
3. Speak your topic when asked by Feynman
4. Teach the concept - speak clearly and wait for silence detection
5. Answer Feynman's probing questions
6. Receive congratulations when complete!

## Technical Details

### Audio Processing
- Frontend captures audio via Web Audio API at 48kHz PCM16
- Audio is encoded to base64 and sent as `input_audio_buffer.append` events
- Silence detection automatically commits audio buffers
- OpenAI Realtime API handles speech-to-text and text-to-speech

### Conversation State Management
- Backend tracks conversation state through 6 phases
- OpenAI responses are parsed to advance conversation state
- State transitions ensure proper conversation flow

### WebSocket Communication
- Browser ↔ Backend: Audio data and control messages
- Backend ↔ OpenAI: Realtime API protocol compliance
- Error handling and reconnection logic included

## Fixed Issues

✅ **OpenAI WebSocket Disconnection**: Fixed audio message formatting to use proper `input_audio_buffer.append` events with base64 encoding instead of raw binary data

✅ **Conversation Flow**: Implemented complete Feynman tutor logic with state management

✅ **Audio Buffer Management**: Added automatic audio commit on silence detection

✅ **UI Feedback**: Enhanced status display and button states

## Testing

The application includes a comprehensive test mode that simulates the complete conversation flow without requiring an OpenAI API key. This allows for development and testing of the conversation logic.