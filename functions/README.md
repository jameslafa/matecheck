# MateCheck Webhook - Firebase Cloud Function

This Cloud Function handles Telegram button callbacks for the snooze feature.

## Setup & Deployment

### 1. Install Firebase CLI (if not already installed)

```bash
npm install -g firebase-tools
```

### 2. Login to Firebase

```bash
firebase login
```

### 3. Install Dependencies

```bash
cd functions
npm install
```

### 4. Set Environment Variables

The function needs your Telegram bot token to answer callback queries:

```bash
firebase functions:config:set telegram.bot_token="YOUR_TELEGRAM_BOT_TOKEN"
```

Replace `YOUR_TELEGRAM_BOT_TOKEN` with the actual token from @BotFather.

### 5. Deploy the Function

```bash
# From the project root directory
firebase deploy --only functions
```

This will:

- Build the TypeScript code
- Deploy to Firebase Cloud Functions
- Give you a webhook URL like: `https://us-central1-matecheck-prod.cloudfunctions.net/webhook`

### 6. Set Telegram Webhook

Once deployed, copy the webhook URL and set it in Telegram:

```bash
curl -X POST "https://api.telegram.org/bot<YOUR_BOT_TOKEN>/setWebhook" \
  -H "Content-Type: application/json" \
  -d '{"url": "https://us-central1-matecheck-prod.cloudfunctions.net/webhook"}'
```

Replace:

- `<YOUR_BOT_TOKEN>` with your actual bot token
- The URL with your actual Cloud Function URL

### 7. Verify Webhook is Set

```bash
curl "https://api.telegram.org/bot<YOUR_BOT_TOKEN>/getWebhookInfo"
```

You should see your webhook URL in the response.

## Testing

1. Run matecheck to send a reminder with buttons
2. Click a snooze button in Telegram
3. You should see:
   - ✅ Confirmation popup in Telegram
   - Friend added to Firestore `snoozes` collection
   - Next day, that friend is filtered from reminders

## Logs

View function logs:

```bash
firebase functions:log
```

## Updating

After making code changes:

```bash
cd functions
npm run build
firebase deploy --only functions
```
