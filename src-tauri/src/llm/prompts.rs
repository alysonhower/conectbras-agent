pub const SYSTEM_MESSAGE: &str = "You are an expert in document analysis and information extraction, specializing in Brazilian business documents. Output only the XML.";

pub const DOCUMENT_PARSE_INITIAL_MESSAGE: &str = r#"You will be given an image of a document page. Your task is to extract the data from this image and structure it as XML. Follow these steps carefully:

1. First, examine the provided document image:
<page_image number="{page_number}">
"#;

pub const DOCUMENT_PARSE_FINAL_MESSAGE: &str = r#"</page_image>

2. Analyze the content of the image. Look for key elements such as:
- Title or heading
- Paragraphs of text
- Lists (bulleted or numbered)
- Tables
- Images or diagrams
- Signatures
- Dates
- Any other relevant information

3. Identify the relationships between these elements. For example, determine which text belongs to which headings, or which cells belong to which rows in a table.

4. Begin structuring the extracted data as XML. Use appropriate tags that describe the content. For example:
- <title> for the main title
- <paragraph> for blocks of text
- <list> for lists, with <item> for each list item
- <table> for tables, with <row> and <cell> for table contents
- <image> for images or diagrams, with a brief description
- <signature> for signatures
- <date> for dates
- Create other tags as necessary to accurately represent the document structure

5. Ensure that your XML structure maintains the hierarchy and relationships of the original document page. Nested elements should be properly indented.

6. If there's any text or content you cannot read or understand clearly, use <unclear> tags to indicate this.

7. Once you've extracted and structured all the data, present your output in the following format:

<page number="{page_number}">
[Your XML-structured data goes here]
</page>

Remember to be as accurate and detailed as possible in your extraction."#;

pub const FILE_NAME_GENERATION_PROMPT: &str = r#"Analyze the following XML representation of a bussiness document to generate an optimal file name for it:

<document>{XML}</document>

In order to create the file name, follow these steps below:

<step number="1">
    Determine the document language. Output your finding within <language> tags as follows:

        <language>[The identified language]</language>
</step>

<step number="2">
    Identify the most relevant date in the document:
    - Format as YYYY-MM-DD
    - Use partial date if full date unavailable
    - Omit if no date is present

    Output your analysis within <important_date> tags as follows:

        <important_date>
            <analysis>[Detailed explanation of your research process and why you chose this specific date as the most relevant]</analysis>
            <date>[The identified date, otherwise leave this tag empty]</date>
        </important_date>
</step>

<step number="3">
    Extract or infer a document type name:
    - Use existing document type names if present
    - If not, create one based on the document content
    - Exclude prepositions and articles

    Output your analysis within <document_type> tags as follows:

        <document_type>
            <analysis>[Detailed explanation of your research process]</analysis>
            <type_name>[The derived document type name]</type_name>
        </document_type>
</step>

<step number="4">
    Extract or derive an abbreviation/initialism for the document type:
    - Use existing abbreviations if present
    - If not, create one based on the document type name
    - Use UPPERCASE letters only (e.g., "Nota Fiscal Eletrônica" -> "NF-E")
    - Exclude prepositions and articles

    Output your analysis within <type_abbreviation> tags as follows:

        <type_abbreviation>
            <analysis>[Detailed explanation of your research process]</analysis>
            <type_abbr>[The derived abbreviation]</type_abbr>
        </type_abbreviation>
</step>

<step number="5">
    Identify the main entities mentioned (if any):
    - Try to find the entity that likely issued the document
    - Try to find the entity to which the document is likely addressed

    Output your analysis within <main_entities> tags as follows:

        <main_entities>
            <analysis>[Detailed explanation of your research process]</analysis>
            <entities>[The identified entities, otherwise leave this tag empty]</entities>
        </main_entities>
</step>

<step number="6">
    Summarize the document's purpose, adhering strictly to these requirements:

    EXTRACTION REQUIREMENTS:
    - MUST match the document language patterns
    - SHOULD elaborate facts from the perspective of the entity to which the document is likely addressed
    - MUST NOT mention the document itself, its type, abbreviation, or the addressed entity
    - PREFER contextualizing data over including raw data explicitly

    FORMATTING REQUIREMENTS:
    - MUST maximize relevant information density within 150 characters
    - MUST employ telegraphic style
    - MUST use abbreviations, initialisms, and short forms for every single word extensively
    - MUST format as lowercase_text_separated_by_underlines_ascii_characters_only
    - MUST NOT use any punctuation marks, accented characters or special characters

    Output your analysis within <document_summary> tags as follows:

        <document_summary>
            <analysis>[Detailed explanation of extraction process and result]</analysis>
            <formatting_process>[Detailed explanation of formatting process]</formatting_process>
            <summary>[The formatted purpose summary]</summary>
        </document_summary>
</step>

<step number="7">
    Build the file name in the following format:

    [YYYY-MM-DD]-[ABBR]-[doc_purp]

    otherwise, if no date present:

    [ABBR]-[doc_purp]

    Where:
    - [YYYY-MM-DD] is the date from step 2
    - [ABBR] is the abbreviation from step 4
    - [doc_purp] is the purpose from step 6

    Finally, provide the file name (without file extension) within <file_name> tags:

        <file_name>[The generated file name]</file_name>
</step>

    Ensure strict adherence to all steps above (particularly the step 6).

Remember to provide your outputs in the same language as the document after determining it in step 1.

<reasoning>
    <language></language>
    <document_type>
        <analysis></analysis>
        <type_name></type_name>
    </document_type>
    <type_abbreviation>
        <analysis></analysis>
        <type_abbr></type_abbr>
    </type_abbreviation>
    <important_date>
        <analysis></analysis>
        <date></date>
    </important_date>
    <main_entities>
        <analysis></analysis>
        <entities></entities>
    </main_entities>
    <document_summary>
        <analysis></analysis>
        <formatting_process></formatting_process>
        <summary></summary>
    </document_summary>
</reasoning>
<file_name></file_name>"#;