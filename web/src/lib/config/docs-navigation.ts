export interface NavItem {
	label: string;
	href: string;
	children?: NavItem[];
}

/**
 * Generate navigation structure from docs directory
 * Updated to match actual files in docs/
 */
export function generateNavigation(): NavItem[] {
	return [
		{
			label: 'Home',
			href: '/'
		},
		{
			label: 'User Guide',
			href: '/user-guide/getting-started',
			children: [
				{
					label: 'Getting Started',
					href: '/user-guide/getting-started'
				},
				{
					label: 'Configuration',
					href: '/user-guide/configuration'
				},
				{
					label: 'Cheatsheet',
					href: '/user-guide/cheatsheet'
				},
				{
					label: 'Cookbook',
					href: '/user-guide/cookbook'
				},
				{
					label: 'Troubleshooting',
					href: '/user-guide/troubleshooting'
				}
			]
		},
		{
			label: 'Tools',
			href: '/tools',
			children: [
				{
					label: 'Overview',
					href: '/tools'
				},
				{
					label: 'inspect_code',
					href: '/tools/inspect_code'
				},
				{
					label: 'search_code',
					href: '/tools/search_code'
				},
				{
					label: 'rename_all',
					href: '/tools/rename_all'
				},
				{
					label: 'relocate',
					href: '/tools/relocate'
				},
				{
					label: 'prune',
					href: '/tools/prune'
				},
				{
					label: 'refactor',
					href: '/tools/refactor'
				},
				{
					label: 'workspace',
					href: '/tools/workspace'
				},
				{
					label: 'system',
					href: '/tools/system'
				}
			]
		},
		{
			label: 'Architecture',
			href: '/architecture',
			children: [
				{
					label: 'Core Concepts',
					href: '/architecture/core-concepts'
				},
				{
					label: 'Specifications',
					href: '/architecture/specifications'
				},
				{
					label: 'Public API',
					href: '/architecture/public_api'
				},
				{
					label: 'Language API',
					href: '/architecture/lang_common_api'
				}
			]
		},
		{
			label: 'Development',
			href: '/development/overview',
			children: [
				{
					label: 'Overview',
					href: '/development/overview'
				},
				{
					label: 'Testing',
					href: '/development/testing'
				},
				{
					label: 'Logging',
					href: '/development/logging_guidelines'
				},
				{
					label: 'Plugin Development',
					href: '/development/plugin-development'
				},
				{
					label: 'Dev Container',
					href: '/development/dev-container'
				}
			]
		},
		{
			label: 'Operations',
			href: '/operations/cache_configuration',
			children: [
				{
					label: 'Cache Config',
					href: '/operations/cache_configuration'
				},
				{
					label: 'CI/CD',
					href: '/operations/cicd'
				},
				{
					label: 'Docker',
					href: '/operations/docker_deployment'
				}
			]
		},
		{
			label: 'Contributing',
			href: '/contributing'
		}
	];
}

/**
 * Get navigation items for a specific section
 */
export function getSectionNav(section: string): NavItem[] {
	const nav = generateNavigation();
	const sectionItem = nav.find((item) => item.href.includes(section));
	return sectionItem?.children || [];
}

export type DocsLink = NavItem;
export const getAllDocsLinks = generateNavigation;
