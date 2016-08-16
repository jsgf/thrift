/*
 * Licensed to the Apache Software Foundation (ASF) under one
 * or more contributor license agreements. See the NOTICE file
 * distributed with this work for additional information
 * regarding copyright ownership. The ASF licenses this file
 * to you under the Apache License, Version 2.0 (the
 * "License"); you may not use this file except in compliance
 * with the License. You may obtain a copy of the License at
 *
 *   http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing,
 * software distributed under the License is distributed on an
 * "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
 * KIND, either express or implied. See the License for the
 * specific language governing permissions and limitations
 * under the License.
 */

#include <map>
#include <fstream>
#include <sstream>
#include <string>
#include <vector>

#include "t_oop_generator.h"
#include "platform.h"
#include "version.h"
#include "logging.h"

using std::map;
using std::ofstream;
using std::string;
using std::vector;

// Which trait implementations to derive; Default and Clone are always generated.
enum Derives {
  DERIVE_DEBUG = 1 << 0,
  DERIVE_HASH = 1 << 1,
  DERIVE_EQ = 1 << 2,
  DERIVE_PARTIALEQ = 1 << 3,
  DERIVE_COPY = 1 << 4,
  DERIVE_ORD = 1 << 5,
  DERIVE_PARTIALORD = 1 << 6,
};

const unsigned DERIVE_ALL =
    DERIVE_DEBUG |
    DERIVE_HASH |
    DERIVE_EQ |
    DERIVE_PARTIALEQ |
    DERIVE_ORD |
    DERIVE_PARTIALORD |
    DERIVE_COPY;

/**
 * Rust code generator.
 */
class t_rs_generator : public t_oop_generator {
 public:
  t_rs_generator(t_program* program,
                 const map<string, string>& parsed_options,
                 const string& option_string)
    : t_oop_generator(program)
  {
    (void) option_string;

    gen_btree_mapset_ = false;
    // FIXME: change back to gen-rs when we finalize mod structure for generated code
    out_dir_base_ = "src";
    for(auto iter = parsed_options.begin(); iter != parsed_options.end(); ++iter) {
      if(iter->first.compare("btree_mapset") == 0) {
        gen_btree_mapset_ = true;
      }
    }
  }

  void init_generator();
  void close_generator();

  /**
   * Program-level generation functions
   */
  void generate_program();
  void generate_typedef(t_typedef*  ttypedef);
  void generate_enum(t_enum*     tenum);
  void generate_struct(t_struct*   tstruct);
  void generate_union(t_struct*   tstruct);
  void generate_service(t_service*  tservice);
  void generate_const(t_const* tconst);

  void print_const_value(std::ofstream& out, std::string name, t_type* type, t_const_value* value);
  std::string render_const_value(std::ofstream& out,
                                 std::string name,
                                 t_type* type,
                                 t_const_value* value);

 private:
  string rs_autogen_comment();
  string rs_imports();

  unsigned rs_type_derives(t_type* type, uint derives);
  string render_rs_type(t_type* type, bool ref = false);
  string render_suffix(t_type* type);
  string render_type_init(t_type* type);

  void generate_service_generics(t_service* tservice);
  void generate_service_fields(t_service* tservice);
  void generate_service_methods(char field, t_service* tservice);
  void generate_service_method_arglist(const vector<t_field*>& fields, bool enumfield);
  void generate_service_uses(t_service* tservice);

  /**
   *Transforms a string with words separated by underscores to a pascal case equivalent
   * e.g. a_multi_word -> AMultiWord
   *      some_name    ->  SomeName
   *      name         ->  Name
   */
  std::string pascalcase(const std::string& in) {
    return capitalize(camelcase(in));
  }

  bool is_string(t_type* type) {
    return type->is_string() && !((t_base_type*)type)->is_binary();
  }

  bool is_binary(t_type* type) {
    return type->is_string() && ((t_base_type*)type)->is_binary();
  }

  static bool is_keyword(const string& id) {
    static string keywords =
      "|abstract|alignof|as|be|box|break|const|continue|crate|do|else|enum|extern|false|final|"
      "fn|for|if|impl|in|let|loop|macro|match|mod|move|mut|offsetof|override|priv|pub|pure|ref|"
      "return|sizeof|static|self|struct|super|true|trait|type|typeof|unsafe|unsized|use|virtual|"
      "where|while|yield|";

    return keywords.find("|" + id + "|") != string::npos;
  }

  static string normalize_id(const string& id) {
    return is_keyword(id) ? id + "_" : id;
  }

  string to_field_name(const string& id) {
    return normalize_id(underscore(id));
  }

 private:
  ofstream f_mod_;
  bool     gen_btree_mapset_;
};

/*
 * Helper class for allocating temp variable names
 */
class t_temp_var {
public:
  t_temp_var() {
    std::stringstream ss;
    // FIXME: are we safe for name clashes?
    ss << "tmp" << index_++;
    name_ = ss.str();
  }
  ~t_temp_var() {
    --index_;
  }
  const string& str() const { return name_; }
private:
  static int index_;
  string name_;
};

int t_temp_var::index_ = 0;


/*
 * This is necessary because we want to generate use clauses for all services,
 */
void t_rs_generator::generate_program() {
  // Initialize the generator
  init_generator();

  // Generate service uses
  vector<t_service*> services = program_->get_services();
  vector<t_service*>::iterator sv_iter;
  for (sv_iter = services.begin(); sv_iter != services.end(); ++sv_iter) {
    generate_service_uses(*sv_iter);
  }

  // Generate enums
  vector<t_enum*> enums = program_->get_enums();
  vector<t_enum*>::iterator en_iter;
  for (en_iter = enums.begin(); en_iter != enums.end(); ++en_iter) {
    generate_enum(*en_iter);
  }

  // Generate typedefs
  vector<t_typedef*> typedefs = program_->get_typedefs();
  vector<t_typedef*>::iterator td_iter;
  for (td_iter = typedefs.begin(); td_iter != typedefs.end(); ++td_iter) {
    generate_typedef(*td_iter);
  }

  // Generate structs, exceptions, and unions in declared order
  vector<t_struct*> objects = program_->get_objects();
  vector<t_struct*>::iterator o_iter;
  for (o_iter = objects.begin(); o_iter != objects.end(); ++o_iter) {
    generate_struct(*o_iter);
  }

  // Generate constants
  vector<t_const*> consts = program_->get_consts();
  vector<t_const*>::iterator c_iter;
  for (c_iter = consts.begin(); c_iter != consts.end(); ++c_iter) {
    generate_const(*c_iter);
  }

  // Generate services
  for (sv_iter = services.begin(); sv_iter != services.end(); ++sv_iter) {
    generate_service(*sv_iter);
  }

  // Close the generator
  close_generator();
}

void t_rs_generator::init_generator() {
  // Make output directory
  // FIXME: enable when finalizing the code structure
  //MKDIR(get_out_dir().c_str());
  string pname = underscore(program_name_);
  string moddirname = get_out_dir() + pname + "/";
  MKDIR(moddirname.c_str());

  // Make output file
  string f_mod_name = moddirname + "mod.rs";
  f_mod_.open(f_mod_name.c_str());

  // Print header
  f_mod_ << rs_autogen_comment() << "\n";
  f_mod_ << rs_imports() << "\n";
}

void t_rs_generator::close_generator() {
  f_mod_.close();
}

string t_rs_generator::rs_autogen_comment() {
  return string(
    "///////////////////////////////////////////////////////////////\n") +
    "// Autogenerated by Thrift Compiler (" + THRIFT_VERSION + ")\n" +
    "//\n" +
    "// DO NOT EDIT UNLESS YOU ARE SURE YOU KNOW WHAT YOU ARE DOING\n" +
    "///////////////////////////////////////////////////////////////\n";
}

string t_rs_generator::rs_imports() {
  return string("#![allow(unused_mut, dead_code, non_snake_case)]\n");
}

// Generates a type alias, translating a thrift `typedef` to a rust `type`.
void t_rs_generator::generate_typedef(t_typedef* ttypedef) {
  string tname = pascalcase(ttypedef->get_symbolic());
  string tdef = render_rs_type(ttypedef->get_type());
  indent(f_mod_) << "pub type " << tname << " = " << tdef << ";\n";
  f_mod_ << "\n";
}

// Generates an enum, translating a thrift enum into a rust enum.
void t_rs_generator::generate_enum(t_enum* tenum) {
  string ename = pascalcase(tenum->get_name());
  indent(f_mod_) << "enom! {\n";
  indent_up();

  indent(f_mod_) << "name = " << ename << ",\n";

  indent(f_mod_) << "values = [\n";
  indent_up();

  // Generate the enum variant declarations.
  vector<t_enum_value*> constants = tenum->get_constants();
  vector<t_enum_value*>::iterator i, end = constants.end();
  for (i = constants.begin(); i != end; ++i) {
    string name = capitalize((*i)->get_name());
    int value = (*i)->get_value();
    indent(f_mod_) << name << " = " << value << ",\n";
  }

  indent_down();
  indent(f_mod_) << "],\n";
  indent(f_mod_) << "default = " << capitalize(constants.at(0)->get_name()) << "\n";

  indent_down();
  indent(f_mod_) << "}\n\n"; // Close enom invocation.
}

static string emit_derives(uint derives) {
  string ret = "";

  if (derives & DERIVE_COPY)
    ret += "Copy, ";
  if (derives & DERIVE_DEBUG)
    ret += "Debug, ";
  if (derives & DERIVE_EQ)
    ret += "Eq, ";
  if (derives & DERIVE_PARTIALEQ)
    ret += "PartialEq, ";
  if (derives & DERIVE_ORD)
    ret += "Ord, ";
  if (derives & DERIVE_PARTIALORD)
    ret += "PartialOrd, ";
  if (derives & DERIVE_HASH)
    ret += "Hash, ";

  return ret;
}

// Generate a struct, translating a thrift struct into a rust struct.
void t_rs_generator::generate_struct(t_struct* tstruct) {
  if (tstruct->is_union()) {
    generate_union(tstruct);
    return;
  }
  string sname = pascalcase(tstruct->get_name());

  indent(f_mod_) << "strukt! {\n";
  indent_up();

  indent(f_mod_) << "name = " << sname << ",\n";

  unsigned derives = rs_type_derives(tstruct, DERIVE_ALL);

  indent(f_mod_) << "derive = [" << emit_derives(derives) << "],\n";

  const vector<t_field*>& members = tstruct->get_members();

  indent(f_mod_) << "reqfields = {\n";
  indent_up();

  for (auto m_iter = members.begin(); m_iter != members.end(); ++m_iter) {
    t_field* tfield = *m_iter;
    if (tfield->get_req() == t_field::T_REQUIRED) {
      auto defl = string("Default::default()");
      if (tfield->get_value() != NULL) {
        defl = render_const_value(f_mod_, tfield->get_name(), tfield->get_type(), tfield->get_value());
      }

      indent(f_mod_) << to_field_name(tfield->get_name())
        << ": " << render_rs_type(tfield->get_type())
        << " => " << tfield->get_key() << ", default = " << defl << ",\n";
    }
  }

  indent_down();
  indent(f_mod_) << "},\n";

  indent(f_mod_) << "optfields = {\n";
  indent_up();

  for (auto m_iter = members.begin(); m_iter != members.end(); ++m_iter) {
    t_field* tfield = *m_iter;
    if (tfield->get_req() != t_field::T_REQUIRED) {
      auto defl = string("Default::default()");
      if (tfield->get_value() != NULL) {
        defl = render_const_value(f_mod_, tfield->get_name(), tfield->get_type(), tfield->get_value()) + ".into()";
      }

      indent(f_mod_) << to_field_name(tfield->get_name())
        << ": " << render_rs_type(tfield->get_type())
        << " => " << tfield->get_key() << ", default = Some(" << defl << "),\n";
    }
  }

  indent_down();
  indent(f_mod_) << "}\n";

  indent_down();
  indent(f_mod_) << "}\n\n"; // Close strukt invocation.
}

// Generate a struct, translating a thrift struct into a rust struct.
void t_rs_generator::generate_union(t_struct* tstruct) {
  if (!tstruct->is_union())
    return;
  string sname = pascalcase(tstruct->get_name());

  indent(f_mod_) << "union! {\n";
  indent_up();

  indent(f_mod_) << "name = " << sname << ",\n";

  unsigned derives = rs_type_derives(tstruct, DERIVE_ALL);

  indent(f_mod_) << "derive = [" << emit_derives(derives) << "],\n";

  const vector<t_field*>& members = tstruct->get_members();

  // At most 1 field can have a default
  string defl = "Unknown";
  for (auto it = members.begin(); it != members.end(); ++it) {
    t_field* tfield = *it;

    if (tfield->get_value() != NULL) {
      string val = render_const_value(f_mod_, tfield->get_name(), tfield->get_type(), tfield->get_value());
      defl = pascalcase(tfield->get_name()) + "(" + val + ".into())";
      break;
    }
  }
  indent(f_mod_) << "default = " << sname << "::" << defl << ",\n";
  indent(f_mod_) << "fields = {\n";
  indent_up();

  for (auto m_iter = members.begin(); m_iter != members.end(); ++m_iter) {
    t_field* tfield = *m_iter;

    indent(f_mod_) << pascalcase(tfield->get_name())
      << ": " << render_rs_type(tfield->get_type())
      << " => " << tfield->get_key() << ",\n";
  }

  indent_down();
  indent(f_mod_) << "}\n";

  indent_down();
  indent(f_mod_) << "}\n\n"; // Close union invocation.
}

// Generate a service, translating from a thrift service to a rust trait.
void t_rs_generator::generate_service(t_service* tservice) {
    const string sname = pascalcase(tservice->get_name());
    const string trait_name = sname;
    const string processor_name = sname + "Processor";
    const string client_name = sname + "Client";

    indent(f_mod_) << "service! {\n";
    indent_up();

    // Trait, processor and client type names.
    indent(f_mod_) << "trait_name = " << trait_name << ",\n";
    indent(f_mod_) << "processor_name = " << processor_name << ",\n";
    indent(f_mod_) << "client_name = " << client_name << ",\n";

    // The methods originating in this service to go in the service trait.
    indent(f_mod_) << "service_methods = [\n";
    indent_up();

    generate_service_methods('a', tservice);

    indent_down();
    indent(f_mod_) << "],\n";

    // The methods from parent services that need to go in the processor.
    indent(f_mod_) << "parent_methods = [\n";
    indent_up();

    char field;
    t_service* parent;
    for (parent = tservice->get_extends(), field = 'b';
         parent && field <= 'z';
         parent = parent->get_extends(), field++) {
        generate_service_methods(field, parent);
    }

    indent_down();
    indent(f_mod_) << "],\n";

    indent(f_mod_) << "bounds = [";
    generate_service_generics(tservice);
    f_mod_ << "],\n";

    indent(f_mod_) << "fields = [";
    generate_service_fields(tservice);
    f_mod_ << "]\n";

    indent_down();
    indent(f_mod_) << "}\n\n";
}

void t_rs_generator::generate_service_methods(char field, t_service* tservice) {
    const string sname = pascalcase(tservice->get_name());

    vector<t_function*> functions = tservice->get_functions();
    vector<t_function*>::const_iterator f_iter;
    for (f_iter = functions.begin(); f_iter != functions.end(); ++f_iter) {
        t_function* tfunction = *f_iter;
        const string argname = sname + pascalcase(tfunction->get_name()) + "Args";
        const string resname = sname + pascalcase(tfunction->get_name()) + "Result";
        const string exnname = sname + pascalcase(tfunction->get_name()) + "Exn";

        indent(f_mod_) << argname << " -> " << resname << " " << exnname << " = "
          << field << "." << tfunction->get_name() << "(\n";

        indent_up();
        generate_service_method_arglist(tfunction->get_arglist()->get_members(), false);
        indent_down();

        indent(f_mod_) << ") -> " << render_rs_type(tfunction->get_returntype()) << " => [\n";

        indent_up();
        generate_service_method_arglist(tfunction->get_xceptions()->get_members(), true);
        indent_down();

        indent(f_mod_) << "],\n";
    }
}

void t_rs_generator::generate_service_generics(t_service* tservice) {
  t_service* parent = tservice;
  char generic = 'A';

  while (parent && generic <= 'Z') {
    f_mod_ << generic << ": " << parent->get_name() << ", ";
    parent = parent->get_extends();
    generic++;
  }
}

void t_rs_generator::generate_service_fields(t_service* tservice) {
  t_service* parent = tservice;
  char generic = 'A';
  char field = 'a';

  while (parent && generic <= 'Z' && field <= 'z') {
    f_mod_ << field << ": " << generic << ", ";
    parent = parent->get_extends();
    generic++;
    field++;
  }
}

void t_rs_generator::generate_service_method_arglist(const vector<t_field*>& fields, bool enumfield) {
    vector<t_field*>::const_iterator field_iter;
    for (field_iter = fields.begin(); field_iter != fields.end(); ++field_iter) {
        t_field* tfield = *field_iter;
        auto field = enumfield ? " " + pascalcase(tfield->get_name()) : "";
        indent(f_mod_) << to_field_name(tfield->get_name())
            << field
            << ": " << render_rs_type(tfield->get_type())
            << " => " << tfield->get_key() << ",\n";
    }
}

void t_rs_generator::generate_service_uses(t_service* tservice) {
  t_service* service = tservice->get_extends();
  while (service) {
    indent(f_mod_) << "use " << service->get_program()->get_name() << "::*;\n";
    service = service->get_extends();
  }
  indent(f_mod_) << "\n";
}

void t_rs_generator::generate_const(t_const* tconst) {
  auto name = tconst->get_name();
  if (uppercase(name) != name) {
    name = uppercase(underscore(tconst->get_name()));
  }
  print_const_value(f_mod_, name, tconst->get_type(), tconst->get_value());
}

// Renders a rust value for a constant
void t_rs_generator::print_const_value(ofstream& out,
                                       string name,
                                       t_type* type,
                                       t_const_value* value) {
  type = get_true_type(type);

  if (type->is_base_type() || type->is_enum()) {
    auto val = render_const_value(out, name, type, value);
    indent(out) << "pub const " << name << ": " << render_rs_type(type, true) << " = " << val << ";\n";
  } else {
    auto val = render_const_value(out, name, type, value);
    indent(out) << "konst! { const " << name << ": " << render_rs_type(type, true) << " = " << val << " }\n";
  }
}

string t_rs_generator::render_const_value(ofstream& out,
                                          string name,
                                          t_type* type,
                                          t_const_value* value) {
  std::ostringstream render;

  type = get_true_type(type);

  if (type->is_base_type()) {
    t_base_type::t_base tbase = ((t_base_type*)type)->get_base();

    switch (tbase) {
    case t_base_type::TYPE_STRING:
      if (((t_base_type*)type)->is_binary())
        render << 'b';
      render << '"' << get_escaped_string(value) << '"';
      break;
    case t_base_type::TYPE_BOOL:
      render << ((value->get_integer() > 0) ? "true" : "false");
      break;
    case t_base_type::TYPE_I8:
    case t_base_type::TYPE_I16:
    case t_base_type::TYPE_I32:
    case t_base_type::TYPE_I64:
      render << value->get_integer();
      break;
    case t_base_type::TYPE_DOUBLE:
      if (value->get_type() == t_const_value::CV_INTEGER) {
        render << value->get_integer();
      } else {
        render << value->get_double();
      }
      break;
    default:
      throw "compiler error: no const of base type " + t_base_type::t_base_name(tbase);
    }
  } else if (type->is_map()) {
    t_type* ktype = ((t_map*)type)->get_key_type();
    t_type* vtype = ((t_map*)type)->get_val_type();

    if (gen_btree_mapset_)
      render << "btreemap_literal! { ";
    else
      render << "hashmap_literal! { ";
    auto map = value->get_map();
    for (auto it = map.begin(); it != map.end(); ++it) {
      auto k = render_const_value(out, name, ktype, it->first);
      auto v = render_const_value(out, name, vtype, it->second);
      render << k << " => " << v << ", ";
    }
    render << "}";
  } else if (type->is_set()) {
    t_type* ty = ((t_set*)type)->get_elem_type();

    if (gen_btree_mapset_)
      render << "btreeset_literal! { ";
    else
      render << "hashset_literal! { ";
    auto set = value->get_list();
    for (auto it = set.begin(); it != set.end(); ++it) {
      auto v = render_const_value(out, name, ty, *it);
      render << v << ", ";
    }
    render << "]";
  } else if (type->is_list()) {
    t_type* ty = ((t_list*)type)->get_elem_type();

    render << "vec! [ ";
    auto list = value->get_list();
    for (auto it = list.begin(); it != list.end(); ++it) {
      auto v = render_const_value(out, name, ty, *it);
      render << v << ", ";
    }
    render << "]";
  } else if (type->is_struct() || type->is_xception()) {
    auto fields = ((t_struct*)type)->get_members();
    auto vals = value->get_map();
    auto sname = pascalcase(type->get_name());

    render << sname << " { ";

    for (auto fit = fields.begin(); fit != fields.end(); ++fit) {

      for (auto vit = vals.begin(); vit != vals.end(); ++vit) {
        if (vit->first->get_string() == (*fit)->get_name()) {
          auto val = render_const_value(out, name, (*fit)->get_type(), vit->second);
          render << (*fit)->get_name() << ": Some(" << val << "), ";
          break;
        }
      }
    }

    render << "..::std::default::Default::default() }";
  } else if (type->is_enum()) {
    auto name = pascalcase(type->get_name());
    render << name << "::" << value->get_identifier_name();
  } else {
    render << "/* Missing thing " << name << " */";
  }

  return render.str();
}

uint t_rs_generator::rs_type_derives(t_type* type, uint derives) {
  type = get_true_type(type);

  if (type->is_base_type()) {
    t_base_type::t_base tbase = ((t_base_type*)type)->get_base();

    switch (tbase) {
      case t_base_type::TYPE_DOUBLE:
        derives &= ~DERIVE_EQ & ~DERIVE_ORD & ~DERIVE_HASH;
        break;
      case t_base_type::TYPE_STRING:
        derives &= ~DERIVE_COPY;
        break;
      default:
        // unchanged
        break;
    }
  } else if (type->is_set()) {
    t_type* etype = ((t_set*)type)->get_elem_type();

    derives &= ~DERIVE_COPY & rs_type_derives(etype, DERIVE_ALL);
    if (!gen_btree_mapset_)
      derives &= ~DERIVE_HASH & ~DERIVE_ORD;
  } else if (type->is_map()) {
    t_type* ktype = ((t_map*)type)->get_key_type();
    t_type* vtype = ((t_map*)type)->get_val_type();

    unsigned kderives = rs_type_derives(ktype, DERIVE_ALL);
    unsigned vderives = rs_type_derives(vtype, DERIVE_ALL);

    derives &= ~DERIVE_COPY;
    if (!gen_btree_mapset_)
      derives &= ~DERIVE_HASH & ~DERIVE_ORD;

    derives &= kderives;
    derives &= vderives;  // XXX overly conservative
  } else if (type->is_list()) {
      t_type* etype = ((t_list*)type)->get_elem_type();

      derives &= ~DERIVE_COPY & rs_type_derives(etype, DERIVE_ALL);
  } else if (type->is_struct() || type->is_xception()) {
      auto fields = ((t_struct*)type)->get_members();

      for (auto fit = fields.begin(); fit != fields.end(); ++fit) {
        derives &= rs_type_derives((*fit)->get_type(), DERIVE_ALL);
      }
  } else if (type->is_enum()) {
    // do nothing
  } else {
    throw "missing unhandled type " + type->get_name();
  }

  return derives;
} 

// Renders a rust type representing the passed in type.
string t_rs_generator::render_rs_type(t_type* type, bool ref) {
  type = get_true_type(type);

  if (type->is_base_type()) {
    t_base_type::t_base tbase = ((t_base_type*)type)->get_base();
    switch (tbase) {
    case t_base_type::TYPE_VOID:
      return "()";
    case t_base_type::TYPE_STRING:
      if (ref)
        return (((t_base_type*)type)->is_binary() ? "&'static [u8]" : "&'static str");
      else
        return (((t_base_type*)type)->is_binary() ? "Vec<u8>" : "String");
    case t_base_type::TYPE_BOOL:
      return "bool";
    case t_base_type::TYPE_I8:
      return "i8";
    case t_base_type::TYPE_I16:
      return "i16";
    case t_base_type::TYPE_I32:
      return "i32";
    case t_base_type::TYPE_I64:
      return "i64";
    case t_base_type::TYPE_DOUBLE:
      return "f64";
    }

  } else if (type->is_enum()) {
    return capitalize(((t_enum*)type)->get_name());

  } else if (type->is_struct() || type->is_xception()) {
    return capitalize(((t_struct*)type)->get_name());

  } else if (type->is_map()) {
    t_type* ktype = ((t_map*)type)->get_key_type();
    t_type* vtype = ((t_map*)type)->get_val_type();
    string maptype;
    if (gen_btree_mapset_)
      maptype = "BTreeMap";
    else
      maptype = "HashMap";

    return "::std::collections::" + maptype + "<" + render_rs_type(ktype, ref) + ", " + render_rs_type(vtype, ref) + ">";

  } else if (type->is_set()) {
    t_type* etype = ((t_set*)type)->get_elem_type();
    string settype;
    if (gen_btree_mapset_)
      settype = "BTreeSet";
    else
      settype = "HashSet";

    return "::std::collections::" + settype + "<" + render_rs_type(etype, ref) + ">";

  } else if (type->is_list()) {
    t_type* etype = ((t_list*)type)->get_elem_type();
    return "Vec<" + render_rs_type(etype, ref) + ">";

  } else {
    throw "INVALID TYPE IN type_to_enum: " + type->get_name();
  }
  return ""; // silence the compiler warning
}

THRIFT_REGISTER_GENERATOR(rs, "Rust",
    "    btree_mapset:     Use BTreeMap/BTreeSet for maps and sets, rather than Hash*\n")

