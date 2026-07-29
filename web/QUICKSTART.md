# Quick Start Guide - Dell G15 AWCC Frontend

Get the advanced web frontend running in minutes!

## ⚡ 60-Second Setup

### 1. Install & Run (Fastest Way)

```bash
cd /vercel/share/v0-project/web
npm install
npm run dev
```

Open [http://localhost:3000](http://localhost:3000) in your browser.

✅ **Done!** You should see the dashboard with live updating metrics.

## 📋 System Requirements

- **Node.js**: 18+ (preferably 20+)
- **npm/yarn**: Any recent version
- **OS**: Linux, macOS, or Windows with WSL

## 🎯 What You Get

✅ Modern dashboard with real-time metrics
✅ Temperature monitoring (CPU/GPU)
✅ Fan speed gauges with circular visualizations
✅ Thermal profile selector
✅ Fan control interface
✅ System status information
✅ Responsive design for all devices
✅ Dark theme with modern UI

## 🖼️ Screenshots

The app displays:
- Real-time CPU/GPU temperatures
- System load visualization
- Fan speed gauges
- Temperature trend charts
- Thermal profile options (Quiet, Balanced, Performance, G-Mode)
- System daemon status
- All animated and responsive

## 🔧 Available Commands

```bash
npm run dev      # Start development server (with hot reload)
npm run build    # Build for production
npm start        # Run production server
npm run lint     # Check code quality
```

## 📱 Device Support

- ✅ Desktop (1920px+)
- ✅ Laptop (1024px+)
- ✅ Tablet (640px+)
- ✅ Mobile (320px+)

All layouts are optimized and responsive.

## 🎨 Design Features

### Visual Elements
- **Dark Theme**: Easy on the eyes
- **Glassmorphism**: Modern semi-transparent cards
- **Gradients**: Blue and orange accent colors
- **Animations**: Smooth transitions throughout
- **Icons**: Professional Lucide icons

### Responsive Grid
- **Mobile**: Single column layout
- **Tablet**: Two-column layout  
- **Desktop**: Three-to-four column layout

## 🔌 Next: Connect to Daemon

The frontend currently uses **mock data**. To connect to the real daemon:

See `INTEGRATION.md` for step-by-step instructions.

Quick preview:
1. Create API routes in `/app/api/`
2. Install `dbus-native` package
3. Implement D-Bus communication
4. Update data fetching with SWR
5. Test with daemon running

## 📁 Project Structure

```
web/
├── app/
│   ├── page.tsx         # Main dashboard
│   ├── layout.tsx       # Root layout
│   └── globals.css      # Styling
├── components/          # 7 reusable components
├── public/              # Static files
├── package.json         # Dependencies
└── tailwind.config.ts   # Tailwind setup
```

## 🚀 Deployment

### Quick Deploy to Vercel

```bash
vercel deploy
```

Or:
1. Push to GitHub
2. Import to Vercel dashboard
3. Deploy with one click

### Self-Hosted

```bash
npm run build
npm start
```

Set `PORT` environment variable if needed:
```bash
PORT=8080 npm start
```

## 🐛 Troubleshooting

### Port 3000 Already in Use

```bash
PORT=3001 npm run dev
```

### Dependencies Installation Failed

```bash
# Clear cache and reinstall
rm -rf node_modules package-lock.json
npm install
```

### Build Errors

```bash
# Check Node version
node --version  # Should be 18+

# Try building again
npm run build
```

### TypeScript Errors

```bash
# Recompile TypeScript
npm run build
```

## 📚 Documentation

- **README.md** - Full project documentation
- **INTEGRATION.md** - Daemon integration guide (step-by-step)
- **CHECKLIST.md** - Implementation checklist
- **FRONTEND_SUMMARY.md** - Feature overview

## 🎯 Common Tasks

### View in Different Viewports

```bash
# Windows/Linux
npm run dev
# Then use browser dev tools (F12) to resize

# macOS
npm run dev
# Cmd+Option+I for dev tools
```

### Inspect Network Requests

1. Open browser dev tools (F12)
2. Go to Network tab
3. Refresh page
4. See all requests

### Enable Dark Mode

Dark theme is always enabled. No toggle needed!

## 💡 Tips

1. **Hot Reload**: Edit files and see changes instantly
2. **Terminal Logs**: Check console for debug messages
3. **Browser Console**: Use Ctrl+Shift+J (Cmd+Option+J on Mac)
4. **Responsive View**: Ctrl+Shift+M (Cmd+Shift+M on Mac)

## 🔑 Key Files to Know

- `app/page.tsx` - Main dashboard page
- `components/` - All UI components
- `app/globals.css` - Theme and global styles
- `tailwind.config.ts` - Tailwind configuration

## 🆘 Need Help?

1. Check `README.md` for detailed info
2. See `INTEGRATION.md` for daemon setup
3. Review error messages carefully
4. Check browser console (F12)

## ✨ Next Steps

After startup, you can:

1. **Explore the Dashboard**: Click around, interact with controls
2. **Read Documentation**: Start with README.md
3. **Set Up Integration**: Follow INTEGRATION.md when ready
4. **Connect Daemon**: Run real data from g15-fancontrold

## 🎓 Learning Resources

- [Next.js Docs](https://nextjs.org)
- [React Docs](https://react.dev)
- [Tailwind CSS](https://tailwindcss.com)
- [Recharts](https://recharts.org)

## 🎉 You're All Set!

Your modern Dell G15 AWCC frontend is ready to use. Enjoy the sleek, responsive dashboard!

---

**Questions?** Check the documentation files or review the code comments.

**Ready for Integration?** See `INTEGRATION.md` for connecting to the daemon.
