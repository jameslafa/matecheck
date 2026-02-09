# MateCheck Web UI Setup

Web interface for managing friends in MateCheck.

## Setup Instructions

### 1. Get Firebase Web App Configuration

1. Go to [Firebase Console](https://console.firebase.google.com/)
2. Select your project: `matecheck-prod`
3. Click the gear icon (Project Settings) → Scroll down to "Your apps"
4. If no web app exists, click "Add app" → Select Web (</>) → Register app
5. Copy the `firebaseConfig` object

You'll get something like:
```javascript
const firebaseConfig = {
  apiKey: "AIza...",
  authDomain: "matecheck-prod.firebaseapp.com",
  projectId: "matecheck-prod",
  storageBucket: "matecheck-prod.firebasestorage.app",
  messagingSenderId: "123456789",
  appId: "1:123456789:web:abc123"
};
```

### 2. Update index.html

Replace the `firebaseConfig` object in `docs/index.html` (around line 391) with your actual config.

### 3. Enable Firebase Authentication

1. Firebase Console → Authentication → Get Started
2. Click "Sign-in method" tab
3. Enable "Google" provider
4. Add your email as an authorized domain (will be added automatically when you deploy)
5. Click "Save"

### 4. Update Firestore Security Rules

Update your Firestore rules to allow web access:

```javascript
rules_version = '2';
service cloud.firestore {
  match /databases/{database}/documents {
    // Snoozes collection
    match /snoozes/{friendId} {
      allow read, write: if request.auth != null;
    }

    // Friends collection
    match /friends/{friendId} {
      allow read, write: if request.auth != null;
    }
  }
}
```

### 5. Enable GitHub Pages

1. Go to your GitHub repo → Settings → Pages
2. Source: Deploy from a branch
3. Branch: `master` (or `main`)
4. Folder: `/docs`
5. Click "Save"

Wait 1-2 minutes for deployment.

Your site will be available at: `https://YOUR_USERNAME.github.io/matecheck/`

### 6. Add GitHub Pages URL to Firebase

1. Firebase Console → Authentication → Settings
2. Under "Authorized domains", click "Add domain"
3. Add: `YOUR_USERNAME.github.io`
4. Click "Add"

## Usage

1. Visit your GitHub Pages URL
2. Click "Sign in with Google"
3. Sign in with the email you used for Firebase
4. You can now:
   - ✅ View all friends
   - ✅ Add new friends
   - ✅ Edit existing friends
   - ✅ Delete friends

Changes take effect immediately in MateCheck!

## Development

To test locally:
1. Run a local server: `python3 -m http.server 8000 --directory docs`
2. Open: `http://localhost:8000`
3. Note: You'll need to add `localhost:8000` to Firebase authorized domains for auth to work

## Security

- Only authenticated users can access the UI
- Authentication is handled by Firebase (Google Sign-in)
- All data is stored in Firestore with security rules
- No sensitive data is exposed in the frontend
