import { Outlet } from 'react-router-dom';

export function LayoutWithHeader() {
  return (
    <div className='bg-primary font-vt323 text-fg-1 min-h-[100vh]'>
      <header></header>
      <main>
        <Outlet />
      </main>
    </div>
  );
}

export function LayoutWithoutHeader() {
  return (
    <div className='bg-primary font-vt323 text-fg-1 min-h-[100vh]'>
      <main>
        <Outlet />
      </main>
    </div>
  );
}
