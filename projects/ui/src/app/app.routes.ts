import { Routes } from '@angular/router';

export const routes: Routes = [
  {
    path: '',
    redirectTo: 'system',
    pathMatch: 'full'
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
    path: 'restore-points',
    loadComponent: () => import('./features/restore-points/restore-points.component')
      .then(m => m.RestorePointsComponent),
    title: 'Restore Points - Syslens'
  },
  {
    path: '**',
    redirectTo: 'system'
  }
];
