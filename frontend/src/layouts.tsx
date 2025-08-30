import { Outlet } from 'react-router-dom';

export function LayoutWithHeader() {
  return (
    <div>
      <header></header>
      <main>
        <Outlet />
      </main>
    </div>
  );
}

export function LayoutWithoutHeader() {
  return (
    <div>
      <main>
        <Outlet />
      </main>
    </div>
  );
}
