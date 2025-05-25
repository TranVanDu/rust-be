-- Delete service items first due to foreign key constraint
DELETE FROM users.service_items WHERE id IN (1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11);

-- Delete services
DELETE FROM users.services WHERE id IN (1, 2, 3, 4, 5); 
