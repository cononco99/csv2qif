NAME
====

**csv2qif** — Generates a quicken readable .qif file based on a .csv file created by exporting schwab transactions.

SYNOPSIS
========

**csv2qif** \[**-c** _current_quicken_securities_file_\] \[**-l** _linked_quicken_cash_account_\] \[_csv_file_from_schwab_\]

DESCRIPTION
===========

Generates three .qif files .   If the _csv_file_from_schwab_ is called foo.CSV then these will have the names:
* investment_transactions_foo.qif
* linked_transactions_foo.qif
* securities_foo.qif

The investment_transactions_foo.qif file will always be generated.  

The linked_transactions_foo.qif file will only be generated if a _linked_quicken_cash_account_ is specified on the command line.  

The securities_foo.qif file will be generated if securities not previously seen in the _current_quicken_securities_file_ are encountered while processing the _csv_file_from_schwab_.


Import these into quicken in this order:

* securities_foo.qif
* investment_transactions_foo.qif
* linked_transactions_foo.qif


When importing the securities_foo.qif file specify "Securities" for importing and import into a non-investment account (perhaps a bank account).

When importing the investment_transactions_foo.qif file specify "Transactions" for importing and import into the appropriate investment account.

When importing the linked_transactions_foo.qif file specify "Transactions" for importing and import into the appropriate linked cash account.
