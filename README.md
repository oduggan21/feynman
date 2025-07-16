# Feynman AI Tutor Setup

## Prerequisites

1. **OpenAI API Key**: You need a valid OpenAI API key with access to the realtime API
2. **Node.js**: For running the frontend
3. **Rust**: For running the backend

## Setup Instructions

### 1. Backend Setup

1. Navigate to the backend directory:
   ```bash
   cd backend
   ```

2. Create a `.env` file with your OpenAI API key:
   ```bash
   echo "OPENAI_API_KEY=your_actual_openai_api_key_here" > .env
   ```
   
   **Important**: Replace `your_actual_openai_api_key_here` with your real OpenAI API key.

3. Install dependencies and run:
   ```bash
   cargo run
   ```

### 2. Frontend Setup

1. Navigate to the frontend directory:
   ```bash
   cd front
   ```

2. Install dependencies:
   ```bash
   npm install
   ```

3. Run the development server:
   ```bash
   npm run dev
   ```

4. Open http://localhost:5173 in your browser

## Usage

1. **Check Connection**: The interface will show your connection status
   - 🟢 Connected: Ready to use
   - 🔴 Disconnected: Check your API key and backend

2. **Start Teaching**: Click the "Start" button and begin speaking to teach Feynman about any topic

3. **Interactive Learning**: Feynman will listen to your explanation and ask probing questions to help identify gaps in your understanding

## Troubleshooting

### "Failed to connect to backend"
- Make sure the backend is running on port 3000
- Check that your OPENAI_API_KEY is valid in the `.env` file

### "Failed to connect to OpenAI"
- Verify your OpenAI API key has access to the realtime API
- Check your internet connection
- Ensure the API key is not expired

### Audio Issues
- Allow microphone access when prompted by your browser
- Use a modern browser that supports Web Audio API