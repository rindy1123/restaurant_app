-- Records in tables and menu_items should be managed by an app for admins.
-- However, for the sake of simplicity, those records are created by this seed file.
INSERT INTO tables (table_number) 
VALUES (1), (2), (3), (4), (5), (6), (7), (8), (9), (10);

INSERT INTO menu_items (name)
VALUES
('Big Mac'),
('Quarter Pounder with Cheese'),
('McChicken'),
('Filet-O-Fish'),
('Chicken McNuggets'),
('McDouble'),
('Egg McMuffin'),
('French Fries');
