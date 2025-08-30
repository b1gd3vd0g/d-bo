import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';

import './style.css';
import { RequireAuth, RequireNoAuth } from './auth_gates';
import { LayoutWithHeader, LayoutWithoutHeader } from './layouts';
import WelcomePage from './pages/welcome';
import { authLoader } from './util/loaders/auth';
import HomePage from './pages/home';
import { createBrowserRouter, RouterProvider } from 'react-router-dom';
import { ConfirmEmailPage, RejectEmailPage } from './pages/email_conf';

const router = createBrowserRouter([
  {
    path: '/',
    children: [
      // These pages can be accessed whether or not there is a player logged in.
      {
        path: 'reject-email',
        element: <RejectEmailPage />
      },
      {
        path: 'confirm-email',
        element: <ConfirmEmailPage />
      },
      // These pages can only be accessed if a player is NOT logged in.
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
      // These pages can only be accessed if a player is logged in.
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
