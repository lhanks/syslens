import { Routes } from '@angular/router';

export const routes: Routes = [
  {
    path: '',
    redirectTo: 'dashboard',
    pathMatch: 'full'
  },
  {
    path: 'dashboard',
    loadComponent: () => import('./features/dashboard/dashboard.component')
      .then(m => m.DashboardComponent),
    title: 'Dashboard - Syslens'
  },
  {
    path: 'network',
    loadComponent: () => import('./features/network/network.component')
      .then(m => m.NetworkComponent),
    title: 'Network - Syslens'
  },
  {
    path: 'system',
    loadComponent: () => import('./features/system/system.component')
      .then(m => m.SystemComponent),
    title: 'System - Syslens'
  },
  {
    path: 'hardware',
    loadComponent: () => import('./features/hardware/hardware.component')
      .then(m => m.HardwareComponent),
    title: 'Hardware - Syslens'
  },
  {
    path: 'storage',
    loadComponent: () => import('./features/storage/storage.component')
      .then(m => m.StorageComponent),
    title: 'Storage - Syslens'
  },
  {
    path: 'processes',
    loadComponent: () => import('./features/processes/processes.component')
      .then(m => m.ProcessesComponent),
    title: 'Processes - Syslens'
  },
  {
    path: '**',
    redirectTo: 'dashboard'
  }
];
