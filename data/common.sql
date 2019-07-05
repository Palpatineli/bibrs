SELECT citation, entry_type, title, booktitle, year, month, chapter, edition, volume, "number", pages
       journals.name
  FROM items
       LEFT JOIN journals ON items.journal_id=journals.id
 WHERE citation = ?

SELECT item_id
  FROM item_persons
       JOIN persons
         ON person_id = persons.id
 WHERE search_term IN ('casagrande', 'rosa')
 GROUP BY item_id
HAVING count(DISTINCT search_term) = 2;
INTERSECT
SELECT item_id
  FROM item_keywords
       JOIN keywords
         ON keyword_id=keywords.id
 WHERE keywords.text IN ()
 GROUP BY item_id
HAVING count(*) = ?;

SELECT items.citation, items.title,
       group_concat(distinct persons.first_name), group_concat(distinct persons.last_name), group_concat(distinct item_persons.order_seq),
       group_concat(distinct keywords.text)
  FROM items
       JOIN item_persons
         ON item_persons.item_id = items.citation
       JOIN persons
         ON item_persons.person_id = persons.id
       JOIN item_keywords
         ON item_keywords.item_id = items.citation
       JOIN keywords
         ON item_keywords.keyword_id = keywords.id
 WHERE items.citation = ?
 GROUP BY items.citation;
