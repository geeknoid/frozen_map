# Frozen Collections

Frozen collections are designed to trade creation time for improved
read performance. They are ideal for use with long-lasting collections
which get initialized when an application starts and remain unchanged
permanently, or at least extended periods of time. This is a common
pattern in service applications.

During creation, the frozen collections perform analysis over the data they
will hold to determine the best layout and algorithm for the specific case.
This analysis can take some time. But the value in spending this time up front
is that the collections provide blazingly fast read-time performance.
