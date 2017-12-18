{
	'start_reading_at' : None,
	'chapter_mark' : u'pagebreak',
	'page_breaks_before' : u"//*[name()='h1' or name()='h2']",
	'remove_first_image' : False,
	'chapter' : u"//*[((name()='h1' or name()='h2') and re:test(., '\\s*((chapter|book|section|part)\\s+)|((prolog|prologue|epilogue)(\\s+|$))', 'i')) or @class = 'chapter']",
	'remove_fake_margins' : True,
	'insert_metadata' : True,
}