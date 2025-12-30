import { Component, inject } from '@angular/core';
import { RouterLink, RouterLinkActive } from '@angular/router';
import { PreloadService } from '@core/services';

interface NavItem {
  label: string;
  route: string;
  icon: string;
}

@Component({
  selector: 'app-top-bar',
  standalone: true,
  imports: [RouterLink, RouterLinkActive],
  template: `
    <header class="h-10 bg-syslens-bg-secondary border-b border-syslens-border-primary flex items-center px-2">
      <nav class="flex items-center gap-1">
        @for (item of navItems; track item.route) {
          <a
            [routerLink]="item.route"
            routerLinkActive="bg-syslens-bg-hover text-syslens-text-primary border-b-2 border-syslens-accent-blue"
            [routerLinkActiveOptions]="{ exact: item.route === '/system' }"
            (mouseenter)="onNavHover(item.route)"
            class="flex items-center gap-1.5 px-3 py-1.5 rounded-t text-sm text-syslens-text-secondary
                   hover:bg-syslens-bg-hover hover:text-syslens-text-primary transition-colors"
          >
            <span class="w-4 h-4" [innerHTML]="item.icon"></span>
            <span>{{ item.label }}</span>
          </a>
        }
      </nav>
    </header>
  `,
  styles: [`
    :host {
      display: block;
    }

    a.border-b-2 {
      margin-bottom: -1px;
    }
  `]
})
export class TopBarComponent {
  private preloadService = inject(PreloadService);

  navItems: NavItem[] = [
    {
      label: 'Network',
      route: '/network',
      icon: `<svg fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
          d="M21 12a9 9 0 01-9 9m9-9a9 9 0 00-9-9m9 9H3m9 9a9 9 0 01-9-9m9 9c1.657 0 3-4.03 3-9s-1.343-9-3-9m0 18c-1.657 0-3-4.03-3-9s1.343-9 3-9m-9 9a9 9 0 019-9" />
      </svg>`
    },
    {
      label: 'System',
      route: '/system',
      icon: `<svg fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
          d="M9 3v2m6-2v2M9 19v2m6-2v2M5 9H3m2 6H3m18-6h-2m2 6h-2M7 19h10a2 2 0 002-2V7a2 2 0 00-2-2H7a2 2 0 00-2 2v10a2 2 0 002 2zM9 9h6v6H9V9z" />
      </svg>`
    },
    {
      label: 'Hardware',
      route: '/hardware',
      icon: `<svg fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
          d="M9 3v2m6-2v2M9 19v2m6-2v2M5 9H3m2 6H3m18-6h-2m2 6h-2M7 19h10a2 2 0 002-2V7a2 2 0 00-2-2H7a2 2 0 00-2 2v10a2 2 0 002 2zM9 9h6v6H9V9z" />
      </svg>`
    },
    {
      label: 'Storage',
      route: '/storage',
      icon: `<svg fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
          d="M4 7v10c0 2.21 3.582 4 8 4s8-1.79 8-4V7M4 7c0 2.21 3.582 4 8 4s8-1.79 8-4M4 7c0-2.21 3.582-4 8-4s8 1.79 8 4m0 5c0 2.21-3.582 4-8 4s-8-1.79-8-4" />
      </svg>`
    },
    {
      label: 'Processes',
      route: '/processes',
      icon: `<svg fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
          d="M4 6h16M4 10h16M4 14h16M4 18h16" />
      </svg>`
    },
    {
      label: 'Services',
      route: '/services',
      icon: `<svg fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
          d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
          d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
      </svg>`
    },
    {
      label: 'Restore',
      route: '/restore-points',
      icon: `<svg fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
          d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" />
      </svg>`
    }
  ];

  onNavHover(route: string): void {
    const view = route.replace('/', '') || 'system';
    this.preloadService.priorityPreload(view);
  }
}
