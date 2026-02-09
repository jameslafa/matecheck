# Quick Setup Guide

## Step 1: Enable Google Authentication

1. Go to: https://console.firebase.google.com/project/matecheck-prod/authentication/providers
2. Click "Get started" if you haven't set up authentication yet
3. Click "Google" in the list of providers
4. Toggle "Enable"
5. Choose a support email (your email)
6. Click "Save"

## Step 2: Add Authorized Domains (for local testing)

1. In Firebase Console → Authentication → Settings
2. Scroll to "Authorized domains"
3. Click "Add domain"
4. Add: `localhost`
5. Click "Add"

## Step 3: Test Locally

1. **Server is already running at:** http://localhost:8000
2. Open that URL in your browser
3. Click "Sign in with Google"
4. Sign in with your Google account
5. You should see your friends list!

## Step 4: Deploy to GitHub Pages

Once local testing works, commit and push:

```bash
git add docs/
git commit -m "Add web UI for friends management"
git push
```

Then enable GitHub Pages:
1. Go to: https://github.com/jameslafa/matecheck/settings/pages
2. Source: Deploy from a branch
3. Branch: `master`
4. Folder: `/docs`
5. Click "Save"

Your site will be at: https://jameslafa.github.io/matecheck/

## Step 5: Add GitHub Pages to Authorized Domains

1. Firebase Console → Authentication → Settings → Authorized domains
2. Click "Add domain"
3. Add: `jameslafa.github.io`
4. Click "Add"

Done! 🎉
