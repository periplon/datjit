/// Minimal embedded corpus for zero-config operation.
/// This provides fallback data when no external corpus is available.

pub static FIRST_NAMES_MALE: &[&str] = &[
    "James", "John", "Robert", "Michael", "David", "William", "Richard", "Joseph", "Thomas",
    "Christopher", "Charles", "Daniel", "Matthew", "Anthony", "Mark", "Donald", "Steven", "Paul",
    "Andrew", "Joshua", "Kenneth", "Kevin", "Brian", "George", "Timothy", "Ronald", "Edward",
    "Jason", "Jeffrey", "Ryan", "Jacob", "Gary", "Nicholas", "Eric", "Jonathan", "Stephen",
    "Larry", "Justin", "Scott", "Brandon", "Benjamin", "Samuel", "Raymond", "Gregory", "Frank",
    "Alexander", "Patrick", "Jack", "Dennis", "Jerry",
];

pub static FIRST_NAMES_FEMALE: &[&str] = &[
    "Mary", "Patricia", "Jennifer", "Linda", "Barbara", "Elizabeth", "Susan", "Jessica", "Sarah",
    "Karen", "Lisa", "Nancy", "Betty", "Margaret", "Sandra", "Ashley", "Dorothy", "Kimberly",
    "Emily", "Donna", "Michelle", "Carol", "Amanda", "Melissa", "Deborah", "Stephanie", "Rebecca",
    "Sharon", "Laura", "Cynthia", "Kathleen", "Amy", "Angela", "Shirley", "Anna", "Brenda",
    "Pamela", "Emma", "Nicole", "Helen", "Samantha", "Katherine", "Christine", "Debra", "Rachel",
    "Carolyn", "Janet", "Catherine", "Maria", "Heather",
];

pub static LAST_NAMES: &[&str] = &[
    "Smith", "Johnson", "Williams", "Brown", "Jones", "Garcia", "Miller", "Davis", "Rodriguez",
    "Martinez", "Hernandez", "Lopez", "Gonzalez", "Wilson", "Anderson", "Thomas", "Taylor",
    "Moore", "Jackson", "Martin", "Lee", "Perez", "Thompson", "White", "Harris", "Sanchez",
    "Clark", "Ramirez", "Lewis", "Robinson", "Walker", "Young", "Allen", "King", "Wright",
    "Scott", "Torres", "Nguyen", "Hill", "Flores", "Green", "Adams", "Nelson", "Baker", "Hall",
    "Rivera", "Campbell", "Mitchell", "Carter", "Roberts", "Kim", "Chen", "Wang", "Patel",
    "Singh", "Li", "Zhang", "Liu", "Yang", "Wu",
];

pub static CITIES: &[(&str, &str, &str, &str)] = &[
    ("New York", "NY", "10001", "America/New_York"),
    ("Los Angeles", "CA", "90001", "America/Los_Angeles"),
    ("Chicago", "IL", "60601", "America/Chicago"),
    ("Houston", "TX", "77001", "America/Chicago"),
    ("Phoenix", "AZ", "85001", "America/Phoenix"),
    ("Philadelphia", "PA", "19101", "America/New_York"),
    ("San Antonio", "TX", "78201", "America/Chicago"),
    ("San Diego", "CA", "92101", "America/Los_Angeles"),
    ("Dallas", "TX", "75201", "America/Chicago"),
    ("San Jose", "CA", "95101", "America/Los_Angeles"),
    ("Austin", "TX", "73301", "America/Chicago"),
    ("Jacksonville", "FL", "32099", "America/New_York"),
    ("Fort Worth", "TX", "76101", "America/Chicago"),
    ("Columbus", "OH", "43085", "America/New_York"),
    ("Charlotte", "NC", "28201", "America/New_York"),
    ("Indianapolis", "IN", "46201", "America/New_York"),
    ("San Francisco", "CA", "94101", "America/Los_Angeles"),
    ("Seattle", "WA", "98101", "America/Los_Angeles"),
    ("Denver", "CO", "80201", "America/Denver"),
    ("Nashville", "TN", "37201", "America/Chicago"),
    ("Portland", "OR", "97201", "America/Los_Angeles"),
    ("Las Vegas", "NV", "89101", "America/Los_Angeles"),
    ("Memphis", "TN", "38101", "America/Chicago"),
    ("Louisville", "KY", "40201", "America/New_York"),
    ("Baltimore", "MD", "21201", "America/New_York"),
    ("Milwaukee", "WI", "53201", "America/Chicago"),
    ("Albuquerque", "NM", "87101", "America/Denver"),
    ("Tucson", "AZ", "85701", "America/Phoenix"),
    ("Fresno", "CA", "93701", "America/Los_Angeles"),
    ("Sacramento", "CA", "95801", "America/Los_Angeles"),
    ("Mesa", "AZ", "85201", "America/Phoenix"),
    ("Atlanta", "GA", "30301", "America/New_York"),
    ("Kansas City", "MO", "64101", "America/Chicago"),
    ("Omaha", "NE", "68101", "America/Chicago"),
    ("Colorado Springs", "CO", "80901", "America/Denver"),
    ("Raleigh", "NC", "27601", "America/New_York"),
    ("Miami", "FL", "33101", "America/New_York"),
    ("Minneapolis", "MN", "55401", "America/Chicago"),
    ("Tampa", "FL", "33601", "America/New_York"),
    ("New Orleans", "LA", "70112", "America/Chicago"),
    ("Cleveland", "OH", "44101", "America/New_York"),
    ("Pittsburgh", "PA", "15201", "America/New_York"),
    ("Cincinnati", "OH", "45201", "America/New_York"),
    ("St. Louis", "MO", "63101", "America/Chicago"),
    ("Orlando", "FL", "32801", "America/New_York"),
    ("Detroit", "MI", "48201", "America/New_York"),
    ("Boston", "MA", "02101", "America/New_York"),
    ("Honolulu", "HI", "96801", "Pacific/Honolulu"),
    ("Salt Lake City", "UT", "84101", "America/Denver"),
    ("Richmond", "VA", "23219", "America/New_York"),
];

pub static COMPANY_PREFIXES: &[&str] = &[
    "Apex", "Meridian", "Nova", "Atlas", "Zenith", "Vertex", "Pulse", "Nexus", "Horizon",
    "Summit", "Stellar", "Quantum", "Dynamo", "Vanguard", "Pinnacle", "Catalyst", "Fusion",
    "Matrix", "Axiom", "Prism",
];

pub static COMPANY_CORES: &[&str] = &[
    "Tech", "Data", "Systems", "Solutions", "Digital", "Cloud", "Labs", "Logic", "Works",
    "Dynamics", "Intelligence", "Analytics", "Insights", "Networks", "Innovations",
];

pub static COMPANY_SUFFIXES: &[&str] = &[
    "Inc.", "Corp.", "LLC", "Labs", "Group", "Co.", "Partners", "Ltd.", "Technologies",
    "Enterprises",
];

pub static JOB_TITLES: &[(&str, &str)] = &[
    ("Software Engineer", "Engineering"),
    ("Senior Software Engineer", "Engineering"),
    ("Product Manager", "Product"),
    ("Data Scientist", "Data"),
    ("UX Designer", "Design"),
    ("DevOps Engineer", "Engineering"),
    ("Marketing Manager", "Marketing"),
    ("Sales Representative", "Sales"),
    ("Customer Success Manager", "Customer Success"),
    ("Business Analyst", "Operations"),
    ("Project Manager", "Operations"),
    ("QA Engineer", "Engineering"),
    ("Frontend Developer", "Engineering"),
    ("Backend Developer", "Engineering"),
    ("Full Stack Developer", "Engineering"),
    ("Machine Learning Engineer", "Data"),
    ("Technical Writer", "Documentation"),
    ("Security Engineer", "Security"),
    ("Database Administrator", "Engineering"),
    ("Cloud Architect", "Engineering"),
];

pub static EMAIL_DOMAINS: &[(&str, f64)] = &[
    ("gmail.com", 30.0),
    ("yahoo.com", 10.0),
    ("outlook.com", 10.0),
    ("hotmail.com", 5.0),
    ("icloud.com", 5.0),
    ("protonmail.com", 3.0),
    ("example.com", 20.0),
    ("test.org", 10.0),
    ("company.com", 7.0),
];

pub static STREET_SUFFIXES: &[&str] = &[
    "St", "Ave", "Blvd", "Dr", "Ln", "Way", "Ct", "Pl", "Rd", "Cir",
];

pub static STREET_NAMES: &[&str] = &[
    "Main", "Oak", "Park", "Elm", "Cedar", "Maple", "Pine", "Walnut", "Washington", "Lake",
    "Hill", "Spring", "Church", "Forest", "River", "Sunset", "Highland", "Meadow", "Valley",
    "Garden",
];
