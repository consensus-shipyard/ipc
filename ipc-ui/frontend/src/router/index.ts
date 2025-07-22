import { createRouter, createWebHistory } from 'vue-router'
import DashboardView from '../views/DashboardView.vue'

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    {
      path: '/',
      name: 'dashboard',
      component: DashboardView,
      meta: { title: 'Dashboard' }
    },
    {
      path: '/wizard',
      name: 'wizard',
      component: () => import('../views/WizardView.vue'),
      meta: { title: 'Deploy New Subnet' },
      children: [
        {
          path: '',
          redirect: '/wizard/template'
        },
        {
          path: 'template',
          name: 'wizard-template',
          component: () => import('../views/wizard/TemplateSelectionView.vue'),
          meta: { title: 'Select Template', step: 1 }
        },
        {
          path: 'basic',
          name: 'wizard-basic',
          component: () => import('../views/wizard/BasicConfigView.vue'),
          meta: { title: 'Basic Configuration', step: 2 }
        },
        {
          path: 'advanced',
          name: 'wizard-advanced',
          component: () => import('../views/wizard/AdvancedConfigView.vue'),
          meta: { title: 'Advanced Settings', step: 3 }
        },
        {
          path: 'activation',
          name: 'wizard-activation',
          component: () => import('../views/wizard/ActivationConfigView.vue'),
          meta: { title: 'Activation Configuration', step: 4 }
        },
        {
          path: 'review',
          name: 'wizard-review',
          component: () => import('../views/wizard/ReviewDeployView.vue'),
          meta: { title: 'Review & Deploy', step: 5 }
        },
        {
          path: 'deploy',
          name: 'wizard-deploy',
          component: () => import('../views/wizard/DeployProgressView.vue'),
          meta: { title: 'Deployment Progress', step: 6 }
        }
      ]
    },
    {
      path: '/instance/:id',
      name: 'instance-detail',
      component: () => import('../views/InstanceDetailView.vue'),
      meta: { title: 'Subnet Details' },
      props: true
    },
    {
      path: '/settings',
      name: 'settings',
      component: () => import('../views/SettingsView.vue'),
      meta: { title: 'Settings' }
    },
    // Redirect any unknown routes to dashboard
    {
      path: '/:pathMatch(.*)*',
      redirect: '/'
    }
  ],
})

export default router
