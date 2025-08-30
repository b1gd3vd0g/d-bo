import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';

import './style.css';
import { RequireAuth, RequireNoAuth } from './auth_gates';
import { LayoutWithHeader, LayoutWithoutHeader } from './layouts';
import WelcomePage from './pages/welcome';
import { authLoader } from './util/loaders/auth';
import HomePage from './pages/home';
import { createBrowserRouter, RouterProvider } from 'react-router-dom';

const router = createBrowserRouter([
  {
    path: '/',
    children: [
      {
        element: <RequireNoAuth />,
        children: [
          {
            element: <LayoutWithoutHeader />,
            children: [
              {
                path: 'welcome',
                element: <WelcomePage />
              }
            ]
          }
        ]
      },
      {
        element: <RequireAuth />,
        loader: authLoader,
        children: [
          {
            element: <LayoutWithHeader />,
            children: [
              {
                index: true,
                element: <HomePage />
              }
            ]
          }
        ]
      }
    ]
  }
]);

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <RouterProvider router={router} />
  </StrictMode>
);
