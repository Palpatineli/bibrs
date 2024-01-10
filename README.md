`bibrs`, a fast bibliography management tool on the command line.
====================================

## What does it do?

`bibrs` manages paper/chapter information and related pdf files. Below I'm referring to all entries as 'papers', even though this should work for chapters and proceedings equally well.

## Add new paper

`bibrs a [KEYWORD(s),]`

1. Download a scholar.bib into your download folder (configurable as `temp_bib`.folder, default to ~/Downloads)
    - The name is fixed, needs to be scholar.bib, as it is the default in google scholar
2. Download the associated pdf file to the pdf download folder (configurable as temp_pdf.folder, default to ~/Downloads)
3. Run `bibrs a [KEYWORD(s)]` with the series of keywords separated by commas.
4. An ID will be generated for the paper, which is first author name + year, for example, `watson1953`.
    1. If IDs starting with such an ID already exist (either `watson1953`, or `watson1953a`):
        1. A prompt gives a numbered list of existing entries
        2. Asks if you want to modify an existing entry, or create a new one
        3. If modifying an existing entry, enter the number of the existing entry, go to step 5 where the indicated entry is modified
        4. If creating a new entry, enter a suffix made of an alphabetic sequence.
            1. If the suffix exists, go back to step 4.1.1
            2. If the suffix is new, go to step 6 where a new entry is created
    2. If the ID has not been used, directly go to step 6 where a new entry is created
5. The entry is modified, using the information from the scholar.bib file, the following fields have special treatment
    1. For each author
        1. If it matches an existing author by both first and last name, use the existing author.
        2. If the last name doesn't match any existing author, use the first and last name from the new paper.
        3. If some existing authors with the same last name as the new one but none of their first names match, print a numbered list of existing first names
        4. Prompt: select an existing first name, use the new name by writing `n`, or write the new first name
    2. Chapter | volume | issue | year are to be coerced to integers
    3. The journal name is searched in the database
        1. Use exact matches if exists for either full name, or abbreviated
        2. If doesn't exist, need the journal's full name, abbreviated name, abbreviated name without dots, separated by commas
    4. The comment file (if exists) is kept untouched. The pdf file is updated with the most recent pdf file in the `temp_pdf` folder if exists.
    5. Keywords will only be added and not removed in this modification process. To remove keywords, directly manipulate keywords for the paper.
6. A new entry is added using the information from the scholar.bib file, following the same routine as 5.1 ~ 5.3
    1. If a pdf file exists in the `temp_pdf.folder`, move it to `pdf.folder`

## Delete a paper

`bibrs d ID`

1. remove the entry, delete the associated pdf file and the comment file
2. remove authors if they only appear for this paper

## Search for paper

`bibrs s [-a AUTHOR] | [-k KEYWORD]`

1. Search for papers written by author's last name, and with keywords
2. The result has both the ID and basic reference, ordered in by year and ID
