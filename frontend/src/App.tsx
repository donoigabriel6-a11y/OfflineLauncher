// Main App component with routing
import { Suspense, lazy } from 'react';
import { BrowserRouter, Routes, Route, Navigate } from 'react-router-dom';
import { Layout } from '@/components/layout/Layout';
import { ToastProvider } from '@/components/ui/Toaster';

// Lazy load pages for code splitting
const Home = lazy(() => import('@/pages/Home').then((module) => ({ default: module.Home })));
const Accounts = lazy(() =>
  import('@/pages/Accounts').then((module) => ({ default: module.Accounts }))
);
const Versions = lazy(() =>
  import('@/pages/Versions').then((module) => ({ default: module.Versions }))
);
const Instances = lazy(() =>
  import('@/pages/Instances').then((module) => ({ default: module.Instances }))
);
const Settings = lazy(() =>
  import('@/pages/Settings').then((module) => ({ default: module.Settings }))
);

// Loading fallback
function PageLoading() {
  return (
    <div className="flex h-64 items-center justify-center">
      <div className="h-8 w-8 animate-spin rounded-full border-2 border-primary-500 border-t-transparent" />
    </div>
  );
}

function App() {
  return (
    <ToastProvider>
      <BrowserRouter>
        <Routes>
          <Route path="/" element={<Layout />}>
            <Route index element={<Navigate to="/home" replace />} />
            <Route
              path="home"
              element={
                <Suspense fallback={<PageLoading />}>
                  <Home />
                </Suspense>
              }
            />
            <Route
              path="accounts"
              element={
                <Suspense fallback={<PageLoading />}>
                  <Accounts />
                </Suspense>
              }
            />
            <Route
              path="versions"
              element={
                <Suspense fallback={<PageLoading />}>
                  <Versions />
                </Suspense>
              }
            />
            <Route
              path="instances"
              element={
                <Suspense fallback={<PageLoading />}>
                  <Instances />
                </Suspense>
              }
            />
            <Route
              path="settings"
              element={
                <Suspense fallback={<PageLoading />}>
                  <Settings />
                </Suspense>
              }
            />
          </Route>
          {/* Fallback */}
          <Route path="*" element={<Navigate to="/home" replace />} />
        </Routes>
      </BrowserRouter>
    </ToastProvider>
  );
}

export default App;
