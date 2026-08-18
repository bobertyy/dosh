ALTER TABLE accounts DROP CONSTRAINT accounts_class_check;

-- A row stored before subclasses existed names no subclass, so each is moved to
-- the most general subclass of the class it already had.
UPDATE accounts SET class = 'asset.current'     WHERE class = 'asset';
UPDATE accounts SET class = 'expense.general'   WHERE class = 'expense';
UPDATE accounts SET class = 'liability.current' WHERE class = 'liability';
UPDATE accounts SET class = 'revenue.sales'     WHERE class = 'revenue';

ALTER TABLE accounts ADD CONSTRAINT accounts_class_check CHECK (class IN (
    'asset.bank',
    'asset.current',
    'asset.fixed',
    'asset.inventory',
    'asset.non_current',
    'asset.prepayment',
    'equity',
    'expense.depreciation',
    'expense.direct_costs',
    'expense.general',
    'expense.overhead',
    'liability.current',
    'liability.non_current',
    'revenue.other_income',
    'revenue.sales'
));
