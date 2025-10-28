## Inputs 
- vendor sku
- upc
- cost
- suggested retail
- description
- weight

## Steps 
- Go to F10-I
- Select Item # field (should be default)
- F6
- Enter upc up to but not including the last digit 
    - ABC does not always save the last digit of UPC, so including may hide valid results 
- One of several things may happen here:
    - No search results appear:
        - Mark the line item as new and return it later 
    - Multiple search result appear:
        - Mark the line item as duplicate and return it later 
    - One search result appears:
        - Select the result and "ENTER"
- If any of the following are true, mark the line as odd and return it later:
    - The cost has changed by a factor of 2 or more 
    - The list price has changed by a factor of 2 or more 
- If the item number or one of the alt skus matches the vendor sku, continue. Otherwise:
    - Pick an empty spot in the alt skus and add a line with the vendor sku 
- If the weight in ABC does not match the weight from the vendor:
    - Enter the vendor provided weight
- If there is no "Group" in ABC:
    - enter Group of "Z"
- If the cost in ABC does not match the vendor cost:
    - Enter the vendor provided cost
- If the list in ABC does not match the vendor list:
    - If the cost has gone down or stayed the same, change nothing

## Todo 
- [x] Only mark a product as a duplicate if the "duplicates" have different skus 
- [x] fix alt skus for existing products
- [ ] try looking up products by alt skus
