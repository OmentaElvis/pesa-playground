import { z } from 'zod';
import { SimulationMode } from './api';

// Zod Schema for client-side validation
export const newProjectSchema = z
	.object({
		projectName: z.string().min(3, 'Project name is required.'),
		businessChoice: z.enum(['existing', 'new']),
		existingBusinessId: z.string().optional(),
		newBusinessName: z.string().optional(),
		newBusinessShortCode: z.string().optional(),
		initialWorkingBalance: z.number().min(1).optional(),
		initialUtilityBalance: z.number().min(0).optional(),
		simulationMode: z.enum(SimulationMode),
		stkDelay: z.number().min(0)
	})
	.superRefine((data, ctx) => {
		if (data.businessChoice == 'new') {
			if (!data.newBusinessName) {
				ctx.addIssue({
					code: 'custom',
					message: 'New business name is required',
					path: ['newBusinessName']
				});
			}

			if (!data.newBusinessShortCode) {
				ctx.addIssue({
					code: 'custom',
					message: 'Shortcode is required',
					path: ['newBusinessShortCode']
				});
			}
		} else if (data.businessChoice == 'existing') {
			if (!data.existingBusinessId) {
				ctx.addIssue({
					code: 'custom',
					message: 'Please select an existing business',
					path: ['existingBusinessId']
				});
			}
		}
	});

export type NewProjectSchema = z.infer<typeof newProjectSchema>;
