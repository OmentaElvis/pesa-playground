import { writable } from 'svelte/store';

export const users = writable([
	{
		id: 1,
		name: 'John Doe',
		phone: '+254712345678',
		balance: 15420.5,
		avatar: 'JD',
		lastActive: '2 minutes ago'
	},
	{
		id: 2,
		name: 'Jane Smith',
		phone: '+254723456789',
		balance: 8750.25,
		avatar: 'JS',
		lastActive: '1 hour ago'
	},
	{
		id: 3,
		name: 'Mike Johnson',
		phone: '+254734567890',
		balance: 23100.75,
		avatar: 'MJ',
		lastActive: '3 hours ago'
	},
	{
		id: 4,
		name: 'Sarah Wilson',
		phone: '+254745678901',
		balance: 5200.0,
		avatar: 'SW',
		lastActive: '5 hours ago'
	},
	{
		id: 5,
		name: 'David Brown',
		phone: '+254756789012',
		balance: 12800.3,
		avatar: 'DB',
		lastActive: '1 day ago'
	}
]);

export const transactions = writable({
	1: [
		{
			id: 1,
			type: 'received',
			amount: 500.0,
			from: 'Jane Smith',
			message:
				'LH7329X5Y2 Confirmed. You have received Ksh500.00 from JANE SMITH 254723456789 on 18/7/2025 at 2:45 PM New M-PESA balance is Ksh15,420.50',
			timestamp: '2:45 PM',
			date: 'Today'
		},
		{
			id: 2,
			type: 'sent',
			amount: 1200.0,
			to: 'NAKUMATT SUPERMARKET',
			message:
				'LH7329X5Y1 Confirmed. Ksh1,200.00 sent to NAKUMATT SUPERMARKET for account 12345 on 18/7/2025 at 1:30 PM New M-PESA balance is Ksh14,920.50',
			timestamp: '1:30 PM',
			date: 'Today'
		},
		{
			id: 3,
			type: 'received',
			amount: 2500.0,
			from: 'Mike Johnson',
			message:
				'LH7329X5Y0 Confirmed. You have received Ksh2,500.00 from MIKE JOHNSON 254734567890 on 18/7/2025 at 10:15 AM New M-PESA balance is Ksh16,120.50',
			timestamp: '10:15 AM',
			date: 'Today'
		},
		{
			id: 4,
			type: 'withdraw',
			amount: 3000.0,
			agent: 'AGENT 12345',
			message:
				'LH7329X5X9 Confirmed. Ksh3,000.00 withdrawn from agent AGENT 12345 on 17/7/2025 at 6:20 PM New M-PESA balance is Ksh13,620.50',
			timestamp: '6:20 PM',
			date: 'Yesterday'
		}
	],
	2: [
		{
			id: 5,
			type: 'sent',
			amount: 800.0,
			to: 'John Doe',
			message:
				'LH7329X5Y3 Confirmed. Ksh800.00 sent to JOHN DOE 254712345678 on 18/7/2025 at 3:15 PM New M-PESA balance is Ksh8,750.25',
			timestamp: '3:15 PM',
			date: 'Today'
		},
		{
			id: 6,
			type: 'airtime',
			amount: 200.0,
			message:
				'LH7329X5Y4 Confirmed. Ksh200.00 airtime bought for 254723456789 on 18/7/2025 at 11:45 AM New M-PESA balance is Ksh8,950.25',
			timestamp: '11:45 AM',
			date: 'Today'
		}
	],
	3: [
		{
			id: 7,
			type: 'deposit',
			amount: 5000.0,
			agent: 'AGENT 67890',
			message:
				'LH7329X5Y5 Confirmed. Ksh5,000.00 deposited to your account from agent AGENT 67890 on 18/7/2025 at 9:30 AM New M-PESA balance is Ksh23,100.75',
			timestamp: '9:30 AM',
			date: 'Today'
		}
	],
	4: [],
	5: [
		{
			id: 8,
			type: 'paybill',
			amount: 1500.0,
			business: 'KENYA POWER',
			message:
				'LH7329X5Y6 Confirmed. Ksh1,500.00 paid to KENYA POWER for account 123456789 on 17/7/2025 at 4:15 PM New M-PESA balance is Ksh12,800.30',
			timestamp: '4:15 PM',
			date: 'Yesterday'
		}
	]
});
