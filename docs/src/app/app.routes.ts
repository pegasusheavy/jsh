import { Routes } from '@angular/router';

export const routes: Routes = [
  {
    path: '',
    loadComponent: () => import('./pages/home/home').then(m => m.HomePage)
  },
  {
    path: 'getting-started',
    loadComponent: () => import('./pages/getting-started/getting-started').then(m => m.GettingStartedPage)
  },
  {
    path: 'bash-zsh',
    loadComponent: () => import('./pages/bash-zsh/bash-zsh').then(m => m.BashZshPage)
  },
  {
    path: 'fish',
    loadComponent: () => import('./pages/fish/fish').then(m => m.FishPage)
  },
  {
    path: 'ash-posix',
    loadComponent: () => import('./pages/ash-posix/ash-posix').then(m => m.AshPosixPage)
  },
  {
    path: 'jsh-syntax',
    loadComponent: () => import('./pages/jsh-syntax/jsh-syntax').then(m => m.JshSyntaxPage)
  },
  {
    path: 'builtins',
    loadComponent: () => import('./pages/builtins/builtins').then(m => m.BuiltinsPage)
  },
  {
    path: '**',
    redirectTo: ''
  }
];
